# cargo build --release
n=0
while [ "$n" -lt 50 ]; do
     n=$(( n + 1 ))
    target/release/simulation-facever \
	--iterations 1 \
	--transfer-size 1024 \
	--interface veth1 \
	--switch-addr 10.0.9.2:1234 \
	--address 10.0.1.2:1234 \
	frontend \
	--fs 10.0.3.2:1234 \
	--storage 10.0.3.2:1234 \
	--gpu 10.0.3.2:1234
done
