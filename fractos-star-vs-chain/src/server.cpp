#include <caladan/api/mo/process.hpp>
#include <caladan/service/global_ns.hpp>

#include "shared.hpp"

using namespace caladan::api::mo;
using namespace caladan;

int main(int argc, char *argv[])
{
  // init logging
  google::InitGoogleLogging(argv[0]);
  google::LogToStderr();
  google::InstallFailureSignalHandler();

  auto proc = process::factory("localhost:3103", "mlx5_0", 1, 0);
  auto ch = proc->channel_create().get();
  auto ep = ch->endpoint_create().get();
  auto gns = service::global_ns::factory().get();

  auto callNotifer = std::promise<void>();
  auto callFuture = callNotifer.get_future();

  auto final_handler = ch->make_request<invocation>(ep)
                           .set_handler([&](auto args)
                                        { 
                                            LOG(INFO) << "In Final Handler";
                                            callNotifer.set_value(); })
                           .create()
                           .get();

  auto server_handler = ch->make_request<invocation>(ep)
                            .set_handler([&](auto args_)
                                         {
                                 std::shared_ptr<typename decltype(args_)::element_type> args = std::move(args_);
                                
                                (void) ch->make_request<invocation>(std::move(args->caps.client_cont))
                                .set_cap(&invocation::caps::client_cont, std::move(args->caps.client_cont))
                                .set_cap(&invocation::caps::server_cont, std::move(args->caps.server_cont))
                                .set_cap(&invocation::caps::end_cont, std::move(args->caps.end_cont)).create().get();

                                return; })
                            .create()
                            .get();

  auto star_server_handler = ch->make_request<invocation>(ep)
                                 .set_handler([&](auto args_)
                                              {
  

                            std::shared_ptr<typename decltype(args_)::element_type> args = std::move(args_);
                          
                          (void) std::move(args->caps.client_cont).invoke().wait();
                          return; })
                                 .create()
                                 .get();

  auto server_revoke = gns->publish_named(ch, ep, server_handler, server_cap_name).get();
  auto star_revoke = gns->publish_named(ch, ep, star_server_handler, star_server_cap_name).get();
  auto final_revoke = gns->publish_named(ch, ep, final_handler, end_cap_name).get();

  LOG(INFO) << "Run Channel";
  ch->run_until([]
                { return false; });
  LOG(INFO) << "Wait for future";
  callFuture.wait();

  LOG(INFO) << "Exiting";
  server_revoke.first.unpublish().get();
  star_revoke.first.unpublish().get();
  final_revoke.first.unpublish().get();

  return EXIT_SUCCESS;
}