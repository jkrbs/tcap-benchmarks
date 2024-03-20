#cargo build # --release
./target/release/denial-of-service \
    --no-packets 10000 \
    --iterations 1 \
    --delay 0 \
    --remote 10.0.1.2:1234 \
    client \
    --interface enp216s0f1 \
    --switch-addr 10.0.9.2:1234 \
    --address 10.0.3.2:1234 
