cargo build --release
n=0
while [ "$n" -lt 50 ]; do
    n=$(( n + 1 ))
    ./target/release/throughput \
    --iterations 50 \
    --remote 10.0.1.2:1234 \
    --scaling \
	client \
	--interface enp94s0f1 \
	--switch-addr 10.0.9.2:1234 \
	--address 10.0.3.2:1234
    sleep 3
done
