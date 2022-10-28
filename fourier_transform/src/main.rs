use num_complex::Complex;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use std::f64::consts::E;
use std::f64::consts::PI;

type Waveform = Vec<Complex<f64>>;

// TODO add parameters for cos() functions
fn create_waveform() -> Waveform {
    let mut waveform = vec![];
    let end = 100;
    let w1 = 2.0 * (PI / (end as f64));
    let a1 = 10 as f64;
    let w2 = 4.0 * (PI / (end as f64));
    let a2 = 5 as f64;
    for i in 0..end {
        let c1 = a1 * (i as f64 * w1).cos();
        let c2 = a2 * (i as f64 * w2).cos();
        waveform.push(Complex::new(c1 + c2, 0.0));
    }
    waveform
}

fn display_waveform(waveform: &Waveform) {
    let mut data_re = vec![];
    let mut data_im = vec![];
    for c in waveform.into_iter().enumerate() {
        data_re.push((c.0 as f64, c.1.re));
        data_im.push((c.0 as f64, c.1.im));
    }
    let pre = Plot::new(data_re).point_style(
        PointStyle::new()
            .marker(PointMarker::Square)
            .colour("#DD3355"),
    );
    let pim = Plot::new(data_im).point_style(PointStyle::new().colour("#35C788"));
    let v = ContinuousView::new().add(pre).add(pim);
    Page::single(&v).save("scatter.svg").unwrap();
    println!("{}", Page::single(&v).dimensions(80, 30).to_text().unwrap());
}

fn fourier_transform(waveform: &Waveform) -> Waveform {
    let const_fac = 2.0 * PI / waveform.len() as f64;
    let mut f = Waveform::new();
    for k in 0..waveform.len() {
        let mut sum = Complex::new(0.0, 0.0);
        for n in 0..waveform.len() {
            let var_fac = k as f64 * n as f64;
            let im = -const_fac * var_fac;
            sum += waveform[n] * Complex::new(E, 0.0).powc(Complex::new(0.0, im));
        }
        f.push(sum);
    }
    f
}

fn main() {
    let waveform = create_waveform();
    display_waveform(&waveform);
    let f = fourier_transform(&waveform);
    display_waveform(&f);
}
