// Runtime configuration injected by the server at startup.
// This file is overwritten in the container entrypoint.
window.__APP_GRAPHQL_ENDPOINT = "http://localhost:3000/graphql";
window.__APP_CONFIG__ = {
    graphqlEndpoint: "http://localhost:3000/graphql",
};
