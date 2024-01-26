cargo build # --release
./target/debug/denial-of-service \
	--no-packets 100000 \
    --iterations 50 \
    --remote 10.0.3.2:1234 \
	--delay 10 \
	server \
	--interface veth1 \
	--switch-addr 10.0.9.2:1234 \
	--address 10.0.1.2:1234
