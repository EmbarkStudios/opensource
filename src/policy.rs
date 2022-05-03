//! Policies for specific projects or users

/// Generally we require that all maintainers of Embark Studios open source projects are
/// part of the Embark org, but this list allows some explicit exceptions
pub const ALLOWED_NON_EMBARK_MAINTAINERS: [&str; 2] = [
    // Emil (https://github.com/emilk) worked at Embark and built 2 open source crates that he continues to co-maintain
    // - https://github.com/EmbarkStudios/puffin
    // - https://github.com/EmbarkStudios/poll-promise
    "emilk",
    // Keith (https://github.com/keith) is contributor-to and co-maintainer of the k8s-buildkite-plugin
    // https://github.com/EmbarkStudios/k8s-buildkite-plugin
    "keith",
];

/// Some project might be public but not quite ready to be listed on the website
pub const IGNORED_PROJECTS: [&str; 1] = [
    // server-framework is still in development (and the name isn't final) so we don't it on the
    // website yet.
    "server-framework",
];
