#cargo build --release
 n=0
 while [ "$n" -lt 50 ]; do
     n=$(( n + 1 ))
    ./target/release/throughput \
    --iterations 1 \
	--remote 10.0.3.2:1234 \
	server \
	--interface enp94s0f0 \
	--switch-addr 10.0.9.2:1234 \
	--address 10.0.1.2:1234
done
