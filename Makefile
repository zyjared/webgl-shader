.PHONY: all clean

all:
	cargo run
	ffmpeg -framerate 30 -i frames/frame_%04d.ppm -y output.gif
ifeq ($(OS),Windows_NT)
	@powershell -Command "Remove-Item -Recurse -Force -ErrorAction SilentlyContinue frames"
else
	@rm -rf frames
endif

clean:
ifeq ($(OS),Windows_NT)
	@powershell -Command "Remove-Item -Recurse -Force -ErrorAction SilentlyContinue output.gif"
else
	@rm -rf output.gif
endif