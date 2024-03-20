# cargo build --release
# n=0
# while [ "$n" -lt 50 ]; do
#     n=$(( n + 1 ))
	# ./target/release/simulation-facever \
	# --iterations 1 \
	# --transfer-size 1 \
	# --interface veth3 \
	# --switch-addr 10.0.9.2:1234 \
	# --address 10.0.3.2:1231 \
	# --debug \
	# fs \
	# --frontend-address 10.0.1.2:1234 &
	# ./target/release/simulation-facever \
	# --iterations 1 \
	# --transfer-size 1 \
	# --interface veth3 \
	# --switch-addr 10.0.9.2:1234 \
	# --address 10.0.3.2:1232 \
	# storage \
	# --frontend-address 10.0.1.2:1234 &
	# ./target/release/simulation-facever \
	# --iterations 1 \
	# --transfer-size 1 \
	# --interface veth3 \
	# --switch-addr 10.0.9.2:1234 \
	# --address 10.0.3.2:1233 \
	# gpu \
	# --frontend-address 10.0.1.2:1234 &
n=0
while [ "$n" -lt 50 ]; do
     n=$(( n + 1 ))
    target/release/simulation-facever \
	--iterations 100 \
	--transfer-size $1 \
	--interface enp94s0f1 \
	--switch-addr 10.0.9.2:1234 \
	--address 10.0.3.2:1234 \
	others \
	--frontend-address 10.0.1.2:1234 
    sleep 3
done
