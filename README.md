<div style="text-align: center; width: 100%;">
    <div style='margin: 0 auto; width: 700px'  >
        <img src="https://raw.githubusercontent.com/gregoriB/mandelbrot-rust/master/mandel.png" alt="mandelbrot pattern"/>
    </div>

Generated with command:
````
$ cargo build --release
$ target/release/mandelbrot mandel.png 4000x3000 -1.20,0.35 -1,0.20
````

Supports adding flag `-st` to the end to run in single threaded mode
</div>
