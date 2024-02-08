#pragma once

#include <caladan/api/mo/cap.hpp>

constexpr const char* server_cap_name = "chain-server";
constexpr const char* star_server_cap_name = "star-server";
constexpr const char* end_cap_name = "chain-end";

struct invocation {
    struct caps {
        caladan::api::mo::cap::request server_cont;
        caladan::api::mo::cap::request client_cont;
        caladan::api::mo::cap::request end_cont;
    };
    struct imms {
    } __attribute((packed));
};