cargo build # --release
./target/debug/denial-of-service \
    --no-packets 1000 \
    --iterations 10 \
    --delay 0 \
    --remote 10.0.1.2:1234 \
    client \
    --interface veth3 \
    --switch-addr 10.0.9.2:1234 \
    --address 10.0.3.2:1234 