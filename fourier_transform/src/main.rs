use num_complex::Complex;
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use std::f64::consts::{E, PI};

type Waveform = Vec<Complex<f64>>;

struct CosParam {
    amplitude: f64,
    frequency: f64,
    phase: f64,
}

impl CosParam {
    fn new(amplitude: i32, frequency: i32, phase: f64) -> CosParam {
        CosParam {
            amplitude: amplitude as f64,
            frequency: frequency as f64,
            phase,
        }
    }
}

fn calc_dcos(param: &CosParam, i: i32, rad_fac: f64) -> f64 {
    param.amplitude * ((i as f64) * param.frequency * rad_fac + param.phase).cos()
}

fn create_waveform(cosparams: &[CosParam], resolution: i32) -> Waveform {
    let rad_fac = 2.0 * PI / resolution as f64;
    let calc_y = |i: i32| {
        let calc_cos = |param: &CosParam| calc_dcos(param, i, rad_fac);
        let re = cosparams.iter().map(calc_cos).sum::<f64>();
        Complex::new(re, 0.0)
    };
    (0..resolution).map(calc_y).collect::<Waveform>()
}

fn create_waveforms() -> Waveform {
    let cps = vec![
        CosParam::new(10, 2, 0.0),
        CosParam::new(15, 3, PI),
        CosParam::new(5, 4, 0.0),
        CosParam::new(7, 5, PI / 2.0),
        CosParam::new(12, 6, PI / 2.0),
        CosParam::new(6, 7, -PI / 2.0),
    ];
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

fn gen_fourier_transform(waveform: &Waveform, negative_exponent: bool, divisor: usize) -> Waveform {
    let neg_fac = if negative_exponent { -1.0 } else { 1.0 };
    let const_fac = neg_fac * 2.0 * PI / waveform.len() as f64;
    let mut f = Waveform::new();
    for k in 0..waveform.len() {
        let mut sum = Complex::new(0.0, 0.0);
        for (n, item) in waveform.iter().enumerate() {
            let var_fac = k as f64 * n as f64;
            let im = const_fac * var_fac;
            sum += item * Complex::new(E, 0.0).powc(Complex::new(0.0, im));
        }
        f.push(sum / divisor as f64);
    }
    f
}

fn fourier_transform(waveform: &Waveform) -> Waveform {
    gen_fourier_transform(waveform, true, 1)
}

fn inverse_fourier_transform(freqform: &Waveform) -> Waveform {
    gen_fourier_transform(freqform, false, freqform.len())
}

fn main() {
    let waveform = create_waveforms();
    display_waveform(&waveform);
    let f = fourier_transform(&waveform);
    display_waveform(&f);
    let waveform_reconstructed = inverse_fourier_transform(&f);
    display_waveform(&waveform_reconstructed);
    // println!("{:?}", f);
}

#[cfg(test)]
mod test {
    use crate::{
        create_waveform, create_waveforms, display_waveform, fourier_transform,
        inverse_fourier_transform, main, CosParam,
    };
    use num_complex::Complex;
    use num_complex::ComplexFloat;
    use std::f64::consts::PI;

    fn calc_open_end(resolution: i32) -> f64 {
        ((resolution - 1) as f64 / resolution as f64 * PI * 2.0).cos()
    }

    #[test]
    fn main_does_not_crash() {
        main();
    }

    #[test]
    fn create_waveforms_creates_multiple_waveforms() {
        let wfs = create_waveforms();
        // some resolution is used
        assert!(1 < wfs.len());
    }

    #[test]
    fn waveform() {
        let cps = vec![CosParam::new(1, 1, 0.0)];
        let resolution = 20;
        let wf = create_waveform(&cps, resolution);

        assert_eq!(resolution as usize, wf.len());
        // cos(0) is 1
        assert_eq!(Complex::new(1.0, 0.0), wf[0]);
        // cos(pi) is -1
        assert_eq!(Complex::new(-1.0, 0.0), wf[wf.len() / 2]);
        // almost cos(2*pi) is 1
        assert_eq!(
            Complex::new(calc_open_end(resolution), 0.0),
            wf[wf.len() - 1]
        );
        // cos(pi/2) is 0
        assert!((Complex::new(0.0, 0.0) - wf[wf.len() / 4]).abs() < f64::EPSILON);
        // cos(pi*3/2) is 0
        assert!((Complex::new(0.0, 0.0) - wf[wf.len() / 4 * 3]).abs() < f64::EPSILON);
    }

    #[test]
    fn resolution_even() {
        let cps = vec![CosParam::new(1, 1, 0.0)];
        let resolution = 2;
        let wf = create_waveform(&cps, resolution);
        assert_eq!(resolution as usize, wf.len());
        assert_eq!(Complex::new(1.0, 0.0), wf[0]);
        assert_eq!(Complex::new(-1.0, 0.0), wf[wf.len() / 2]);
        assert_eq!(
            Complex::new(calc_open_end(resolution), 0.0),
            wf[wf.len() - 1]
        );
    }

    #[test]
    fn resolution_odd() {
        let cps = vec![CosParam::new(1, 1, 0.0)];
        let resolution = 3;
        let wf = create_waveform(&cps, resolution);
        assert_eq!(resolution as usize, wf.len());
        assert_eq!(Complex::new(1.0, 0.0), wf[0]);
        assert_eq!(
            Complex::new(calc_open_end(resolution), 0.0),
            wf[wf.len() - 1]
        );
    }

    #[test]
    fn amplitude() {
        let amplitude = 9;
        let cps = vec![CosParam::new(amplitude, 1, 0.0)];
        let resolution = 20;
        let wf = create_waveform(&cps, resolution);
        assert_eq!(Complex::new(amplitude as f64, 0.0), wf[0]);
        assert_eq!(Complex::new(-amplitude as f64, 0.0), wf[wf.len() / 2]);
        assert_eq!(
            Complex::new(amplitude as f64 * calc_open_end(resolution), 0.0),
            wf[wf.len() - 1]
        );
    }

    #[test]
    fn frequency() {
        let frequency = 4;
        let cps = vec![CosParam::new(1, frequency, 0.0)];
        let resolution = 200;
        let wf = create_waveform(&cps, resolution);
        assert_eq!(Complex::new(1.0, 0.0), wf[0]);
        assert_eq!(Complex::new(-1.0, 0.0), wf[wf.len() / 8]);
        assert_eq!(Complex::new(1.0, 0.0), wf[wf.len() / 4]);
        assert_eq!(Complex::new(-1.0, 0.0), wf[wf.len() * 3 / 8]);
        assert_eq!(Complex::new(1.0, 0.0), wf[wf.len() / 2]);
        assert_eq!(Complex::new(-1.0, 0.0), wf[wf.len() * 5 / 8]);
        assert_eq!(Complex::new(1.0, 0.0), wf[wf.len() * 3 / 4]);
        assert_eq!(Complex::new(-1.0, 0.0), wf[wf.len() * 7 / 8]);
        assert_eq!(
            Complex::new(calc_open_end(resolution / frequency), 0.0),
            wf[wf.len() - 1]
        );
    }

    #[test]
    fn phase() {
        let phase = std::f64::consts::PI;
        let cps = vec![CosParam::new(1, 1, phase)];
        let resolution = 200;
        let wf = create_waveform(&cps, resolution);
        assert_eq!(Complex::new(-1.0, 0.0), wf[0]);
        assert!(f64::EPSILON > wf[wf.len() / 4].abs());
        assert_eq!(Complex::new(1.0, 0.0), wf[wf.len() / 2]);
        assert!(f64::EPSILON * 2.0 > wf[wf.len() * 3 / 4].abs());
        assert_eq!(
            Complex::new(-calc_open_end(resolution), 0.0),
            wf[wf.len() - 1]
        );
    }

    #[test]
    fn fourier_and_inverse() {
        let cps = vec![CosParam::new(1, 1, 0.0)];
        let resolution = 20;
        let wf = create_waveform(&cps, resolution);
        let freqs = fourier_transform(&wf);
        display_waveform(&wf);
        display_waveform(&freqs);
        let wf_recon = inverse_fourier_transform(&freqs);
        display_waveform(&wf_recon);

        for (pos, (orig, recon)) in wf.iter().zip(wf_recon.iter()).enumerate() {
            let diff = (orig - recon).abs();
            println!("{} {}", pos, diff);
            assert!(diff < 19.0 * f64::EPSILON);
        }
    }
}
