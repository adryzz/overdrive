fn main() {
    println!("Hello, world!");
    let a = cvt_rs::CvtTimings::generate(1920, 1080, 60.0, cvt_rs::BlankingMode::ReducedV2, false, false).unwrap();

    dbg!(&a);
    println!("{}", a.generate_modeline());

}
