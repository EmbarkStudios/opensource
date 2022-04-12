//! Policies for specific projects or users

//! Generally we require that all maintainers of Embark Studios open source projects are
//! part of the Embark org, but this list allows some explicit exceptions
pub const ALLOWED_NON_EMBARK_MAINTAINERS: [&str; 1] = [
    // Emil (https://github.com/emilk) worked at Embark and built 2 open source crates that he continues to co-maintain
    // - https://github.com/EmbarkStudios/puffin
    // - https://github.com/EmbarkStudios/poll-promise
    "emilk",
];
