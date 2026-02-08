#!/usr/bin/env ruby
# frozen_string_literal: true

require "yaml"
require "json"
require "fileutils"
require "time"

ROOT = File.expand_path("..", __dir__)

CONFIG_PATH = File.join(ROOT, "arrive", "evidence_scan.yaml")
REGISTRY_PATH = File.join(ROOT, "arrive", "registry.yaml")

unless File.exist?(CONFIG_PATH)
  warn "Missing config: #{CONFIG_PATH}"
  exit 1
end

config = YAML.load_file(CONFIG_PATH)
registry = YAML.load_file(REGISTRY_PATH)

scan_cfg = config.fetch("evidence_scan")
output_dir = File.join(ROOT, scan_cfg.fetch("output_dir"))
FileUtils.mkdir_p(output_dir)

registry_evidence_types = registry.dig("vocabulary", "evidence_types") || []
systems = scan_cfg["systems"] || registry["systems"] || []
default_enforce_links = scan_cfg["enforce_evidence_links"] ? true : false
security_cfg = scan_cfg["security_scan"] || {}
security_required = security_cfg["required"] ? true : false
security_paths = security_cfg["report_paths"] || []
security_reports = security_paths.flat_map do |pattern|
  Dir.glob(File.join(ROOT, pattern)).map { |path| path.sub("#{ROOT}/", "") }
end

promotion_rules = scan_cfg["promotion_rules"] || {}
system_overrides = scan_cfg["systems_overrides"] || {}

now = Time.now.utc
stamp = now.strftime("%Y%m%dT%H%M%SZ")


def load_yaml(path)
  YAML.load_file(path)
end


def parse_frontmatter(md)
  return [{}, md] unless md.start_with?("---\n")
  parts = md.split("---\n", 3)
  return [{}, md] if parts.length < 3
  frontmatter = parts[1]
  body = parts[2]
  [YAML.load(frontmatter) || {}, body]
end


def extract_evidence_section(body)
  lines = body.lines
  idx = lines.find_index { |line| line.strip == "## Evidence" }
  return [] unless idx
  evidence_lines = []
  lines[(idx + 1)..].each do |line|
    break if line.start_with?("## ")
    evidence_lines << line
  end
  evidence_lines
end


def parse_evidence_lines(lines)
  entries = []
  lines.each do |line|
    stripped = line.strip
    next unless stripped.start_with?("-")
    # Example: - [x] tests:unit - Unit tests passing
    if stripped =~ /- \[(x| )\]\s+([^\s]+)\s*(.*)/
      checked = Regexp.last_match(1) == "x"
      evidence_type = Regexp.last_match(2)
      rest = Regexp.last_match(3)
      has_link = rest.include?("http://") || rest.include?("https://") || rest.include?("](")
      entries << {
        "type" => evidence_type,
        "checked" => checked,
        "has_link" => has_link,
        "raw" => stripped
      }
    end
  end
  entries
end


def deep_merge(base, override)
  return base unless override
  merged = base.dup
  override.each do |key, value|
    if value.is_a?(Hash) && base[key].is_a?(Hash)
      merged[key] = deep_merge(base[key], value)
    else
      merged[key] = value
    end
  end
  merged
end

def normalize_evidence_type(item)
  return item if item.is_a?(String)
  if item.is_a?(Hash)
    return item["type"] if item.key?("type")
    if item.length == 1
      key, value = item.first
      return "#{key}:#{value}"
    end
  end
  item.to_s
end

report = {
  "generated_at" => now.iso8601,
  "config" => {
    "path" => "arrive/evidence_scan.yaml",
    "output_dir" => scan_cfg.fetch("output_dir")
  },
  "security_scan" => {
    "required" => security_required,
    "report_paths" => security_paths,
    "reports_found" => security_reports
  },
  "systems" => []
}

systems.each do |system_id|
  system_dir = File.join(ROOT, "arrive", "systems", system_id)
  system_yaml = File.join(system_dir, "system.yaml")
  system_cfg = load_yaml(system_yaml)

  require_links = system_cfg.dig("policies", "resident_gates", "require_evidence_links") ? true : false
  base_rules = { "promotion_rules" => promotion_rules }
  system_rules = deep_merge(base_rules, system_overrides[system_id])
  enforce_links = system_rules.fetch("enforce_evidence_links", default_enforce_links) ? true : false

  components = Dir.glob(File.join(system_dir, "components", "*.yaml")).sort.map do |path|
    data = load_yaml(path)
    comp = data.fetch("component")
    {
      "id" => comp.fetch("id"),
      "name" => comp.fetch("name"),
      "type" => comp.fetch("type"),
      "stage" => comp.fetch("stage"),
      "path" => path.sub("#{ROOT}/", "")
    }
  end

  advances = Dir.glob(File.join(system_dir, "advances", "*.md")).sort.map do |path|
    md = File.read(path)
    frontmatter, body = parse_frontmatter(md)
    advance = frontmatter.fetch("advance", {})
    evidence_lines = extract_evidence_section(body)
    evidence_entries = parse_evidence_lines(evidence_lines)
    {
      "id" => advance["id"],
      "status" => advance["status"],
      "primary_component" => advance["primary_component"],
      "components" => advance["components"] || [],
      "evidence" => advance["evidence"] || [],
      "evidence_entries" => evidence_entries,
      "path" => path.sub("#{ROOT}/", "")
    }
  end

  component_reports = components.map do |component|
    related_advances = advances.select do |adv|
      comps = adv["components"] || []
      adv["primary_component"] == component["id"] || comps.include?(component["id"])
    end

    completed_advances = related_advances.select { |adv| adv["status"] == "complete" }

    evidence_types = []
    evidence_links = {}
    unknown_evidence = []

    completed_advances.each do |adv|
      adv_types = adv["evidence"] || []
      adv_types.each do |etype|
        normalized = normalize_evidence_type(etype)
        evidence_types << normalized
        unknown_evidence << normalized unless registry_evidence_types.include?(normalized)
      end

      adv["evidence_entries"].each do |entry|
        etype = entry["type"]
        evidence_links[etype] ||= []
        evidence_links[etype] << entry if entry["has_link"]
      end
    end

    evidence_types.uniq!
    unknown_evidence.uniq!

    stage = component["stage"]
    stage_rule = system_rules.dig("promotion_rules", stage)

    promotion = {
      "eligible" => false,
      "next_stage" => nil,
      "missing_requirements" => [],
      "missing_evidence_types" => [],
      "missing_evidence_links" => [],
      "missing_evidence_links_warning" => []
    }

    if stage_rule
      promotion["next_stage"] = stage_rule["next_stage"]
      min_advances = stage_rule["min_completed_advances"] || 0
      if completed_advances.length < min_advances
        promotion["missing_requirements"] << "needs #{min_advances} completed advances"
      end
      if security_required && security_reports.empty?
        promotion["missing_requirements"] << "security scan report"
      end
      required_types = stage_rule["required_evidence_types"] || []
      missing_types = required_types.reject { |etype| evidence_types.include?(etype) }
      promotion["missing_evidence_types"] = missing_types

      if require_links
        missing_links = required_types.select do |etype|
          evidence_links.fetch(etype, []).empty?
        end
        if enforce_links
          promotion["missing_evidence_links"] = missing_links
        else
          promotion["missing_evidence_links_warning"] = missing_links
        end
      end

      promotion["eligible"] = promotion["missing_requirements"].empty? &&
        promotion["missing_evidence_types"].empty? &&
        promotion["missing_evidence_links"].empty?
    end

    {
      "component" => component,
      "related_advances" => related_advances.map { |adv| adv["id"] },
      "completed_advances" => completed_advances.map { |adv| adv["id"] },
      "evidence_types" => evidence_types.sort,
      "unknown_evidence_types" => unknown_evidence.sort,
      "promotion" => promotion
    }
  end

  report["systems"] << {
    "id" => system_id,
    "require_evidence_links" => require_links,
    "enforce_evidence_links" => enforce_links,
    "components" => component_reports
  }
end

json_path = File.join(output_dir, "evidence-scan-#{stamp}.json")
File.write(json_path, JSON.pretty_generate(report))

md_lines = []
md_lines << "# ARRIVE Evidence Scan"
md_lines << ""
md_lines << "Generated at: #{now.iso8601}"
md_lines << "Config: arrive/evidence_scan.yaml"
md_lines << "Security scan required: #{report['security_scan']['required'] ? 'yes' : 'no'}"
if report["security_scan"]["report_paths"].any?
  md_lines << "Security scan report paths: #{report['security_scan']['report_paths'].join(', ')}"
end
if report["security_scan"]["reports_found"].empty?
  md_lines << "Security scan reports found: none"
else
  md_lines << "Security scan reports found: #{report['security_scan']['reports_found'].join(', ')}"
end
md_lines << ""

report["systems"].each do |system|
  md_lines << "## #{system['id'].capitalize}"
  md_lines << "Evidence links required by system policy: #{system['require_evidence_links'] ? 'yes' : 'no'}"
  md_lines << "Evidence links enforced by scan config: #{system['enforce_evidence_links'] ? 'yes' : 'no'}"
  md_lines << ""

  system["components"].each do |comp|
    component = comp["component"]
    promotion = comp["promotion"]
    md_lines << "### #{component['id']} (#{component['stage']})"
    md_lines << "- Name: #{component['name']}"
    md_lines << "- Completed advances: #{comp['completed_advances'].length}"
    md_lines << "- Evidence types: #{comp['evidence_types'].empty? ? 'none' : comp['evidence_types'].join(', ')}"
    unless comp['unknown_evidence_types'].empty?
      md_lines << "- Unknown evidence types: #{comp['unknown_evidence_types'].join(', ')}"
    end

    if promotion['next_stage']
      md_lines << "- Promotion target: #{promotion['next_stage']}"
      md_lines << "- Eligible: #{promotion['eligible'] ? 'yes' : 'no'}"
      unless promotion['missing_requirements'].empty?
        md_lines << "- Missing requirements: #{promotion['missing_requirements'].join(', ')}"
      end
      unless promotion['missing_evidence_types'].empty?
        md_lines << "- Missing evidence types: #{promotion['missing_evidence_types'].join(', ')}"
      end
      unless promotion['missing_evidence_links'].empty?
        md_lines << "- Missing evidence links: #{promotion['missing_evidence_links'].join(', ')}"
      end
      unless promotion['missing_evidence_links_warning'].empty?
        md_lines << "- Missing evidence links (not enforced): #{promotion['missing_evidence_links_warning'].join(', ')}"
      end
    else
      md_lines << "- Promotion target: n/a"
    end
    md_lines << ""
  end
end

md_path = File.join(output_dir, "evidence-scan-#{stamp}.md")
File.write(md_path, md_lines.join("\n"))

puts "Wrote #{json_path}"
puts "Wrote #{md_path}"
