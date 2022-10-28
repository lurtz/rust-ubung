use num_complex::Complex;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use std::f64::consts::E;
use std::f64::consts::PI;

type Waveform = Vec<Complex<f64>>;

struct CosParam {
    amplitude: f64,
    frequency: f64,
    phase: f64,
}

impl CosParam {
    fn new(amplitude: i32, frequency: i32, phase: i32) -> CosParam {
        CosParam {
            amplitude: amplitude as f64,
            frequency: frequency as f64,
            phase: phase as f64,
        }
    }
}

fn calc_dcos(param: &CosParam, i: i32, rad_fac: f64) -> f64 {
    // TODO math is really bogus / incoherent
    param.amplitude * ((i as f64) * param.frequency * rad_fac + param.phase).cos()
}

fn create_waveform(cosparams: &[CosParam], resolution: i32) -> Waveform {
    let rad_fac = PI / resolution as f64;
    let calc_y = |i: i32| {
        let calc_cos = |param: &CosParam| calc_dcos(param, i, rad_fac);
        let re = cosparams.iter().map(calc_cos).sum::<f64>();
        Complex::new(re, 0.0)
    };
    (0..resolution)
        .into_iter()
        .map(calc_y)
        .collect::<Waveform>()
}

fn create_waveforms() -> Waveform {
    let cps = vec![CosParam::new(10, 2, 0), CosParam::new(5, 4, 0)];
    create_waveform(&cps, 100)
}

fn display_waveform(waveform: &Waveform) {
    let mut data_re = vec![];
    let mut data_im = vec![];
    for c in waveform.iter().enumerate() {
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
        for (n, item) in waveform.iter().enumerate() {
            let var_fac = k as f64 * n as f64;
            let im = -const_fac * var_fac;
            sum += item * Complex::new(E, 0.0).powc(Complex::new(0.0, im));
        }
        f.push(sum);
    }
    f
}

fn main() {
    let waveform = create_waveforms();
    display_waveform(&waveform);
    let f = fourier_transform(&waveform);
    display_waveform(&f);
}
