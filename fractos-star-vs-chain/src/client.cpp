#include <caladan/api/mo/process.hpp>
#include <caladan/service/global_ns.hpp>

#include "shared.hpp"

#include <ratio>
#include <chrono>

using namespace caladan::api::mo;
using namespace caladan;

void chain(std::shared_ptr<channel> ch, caladan::service::global_ns *gns, uint64_t steps)
{
  auto ep = ch->endpoint_create().get();
  auto callNotifer = std::promise<void>();
  auto callFuture = callNotifer.get_future();

  uint64_t counter = 0;

  auto handler = ch->make_request<invocation>(ep)
                     .set_handler([&](auto args_)
                                  {
                      std::shared_ptr<typename decltype(args_)::element_type> args = std::move(args_);    
                        if (counter == steps) {
                          // call end_cap
                          callNotifer.set_value();
                          return ch->make_request<invocation>(std::move(args->caps.end_cont))
                          .set_cap(&invocation::caps::client_cont, std::move(args->caps.client_cont))
                          .set_cap(&invocation::caps::server_cont, std::move(args->caps.server_cont))
                          .set_cap(&invocation::caps::end_cont, std::move(args->caps.end_cont)).create().get();
                        } else {
                          // call server
                          counter++;
                          return ch->make_request<invocation>(std::move(args->caps.server_cont))
                          .set_cap(&invocation::caps::client_cont, std::move(args->caps.client_cont))
                          .set_cap(&invocation::caps::server_cont, std::move(args->caps.server_cont))
                          .set_cap(&invocation::caps::end_cont, std::move(args->caps.end_cont)).create().get();
                        } })
                     .create()
                     .get();

  auto server = gns->get<cap::request>(ch, ep, server_cap_name).get();
  auto end = gns->get<cap::request>(ch, ep, end_cap_name).get();

  auto req = ch->make_request<invocation>(std::move(server))
                 .set_cap(&invocation::caps::client_cont, std::move(handler))
                 .set_cap(&invocation::caps::server_cont, std::move(server))
                 .set_cap(&invocation::caps::end_cont, std::move(end))
                 .create_and_invoke()
                 .get();

  ch->run_until([&]
                { return callFuture.valid(); });
}

void star(std::shared_ptr<channel> ch, caladan::service::global_ns *gns, uint64_t steps)
{
  auto ep = ch->endpoint_create().get();

  auto server = gns->get<cap::request>(ch, ep, star_server_cap_name).get();

  auto donePromise = std::promise<void>();
  auto done = donePromise.get_future();

  auto ret = std::vector<caladan::api::mo::cap::request>();
  for (int i = 0; i < steps-1; i++)
  {
    ret.push_back(std::move(
      ch->make_request<invocation>(ep).set_handler([](auto args_) {}).create().get()
    ));
  }

  ret.push_back(std::move(
      ch->make_request<invocation>(ep).set_handler([&](auto args_) {
        donePromise.set_value();
      }).create().get()
    ));

  auto handler = ch->make_request<invocation>(ep)
                     .set_handler([&](auto args_)
                                  { 
                                    std::shared_ptr<typename decltype(args_)::element_type> args = std::move(args_); 

                                    for(int i = 0; i < steps; i++) {
                                      (void) server.get_channel()->make_request<invocation>(server).set_cap(&invocation::caps::client_cont, std::move(ret[i])).create().get().invoke().get();
                                    }
                                  })
                     .create_and_invoke()
                     .get();

  ch->run_until([&]
                { return done.valid(); });
}
int main(int argc, char *argv[])
{
  // init logging
  google::InitGoogleLogging(argv[0]);
  google::LogToStderr();
  google::InstallFailureSignalHandler();

  LOG(INFO) << "FractOS initialization ...";
  auto proc = process::factory("localhost:3103", "mlx5_1", 1, 0);
  auto gns = service::global_ns::factory().get();
  LOG(INFO) << "... done";

  std::vector<uint32_t> depthes = {10}; //{10, 100, 1000, 10000, 20000, 40000, 50000, 100000};

  auto times = std::map<uint32_t, std::vector<long>>();
  for (auto depth : depthes)
  {
    times.insert(std::pair<uint32_t, std::vector<long>>(depth, std::vector<long>()));

    for (int i = 0; i < 5; i++)
    {
         auto ch = proc->channel_create().get();
      LOG(INFO) << "Running Chain Benchmark iteration " << i;
      auto start = std::chrono::high_resolution_clock::now();
      chain(ch, gns, depth);
      auto end_time = std::chrono::high_resolution_clock::now();
      const long time = std::chrono::duration_cast<std::chrono::nanoseconds>(end_time - start).count();

      LOG(INFO) << "chain duration in ns: " << time;

      times.at(depth).push_back(time);
    }
  }

  times = std::map<uint32_t, std::vector<long>>();
  for (auto depth : depthes)
  {
    times.insert(std::pair<uint32_t, std::vector<long>>(depth, std::vector<long>()));

    for (int i = 0; i < 5; i++)
    {
      auto ch = proc->channel_create().get();
      //LOG(INFO) << "Running Star Benchmark iteration " << i;
      auto start = std::chrono::high_resolution_clock::now();
      star(std::move(ch), gns, depth);
      auto end_time = std::chrono::high_resolution_clock::now();
      const long time = std::chrono::duration_cast<std::chrono::nanoseconds>(end_time - start).count();
      times.at(depth).push_back(time);
      LOG(INFO) << "chain duration in ns: " << time;
    }
  }

  return EXIT_SUCCESS;
}