video_file := ./assets/ahsoka.mp4

executable := ../target/debug/fql

run:	build
	mv $(executable) .
	./fql $(video_file)

clean:
	rm fql

build:
	-cargo build -p fql
