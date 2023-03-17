fn main() {
    println!("Hello, world!");
    dbg!(cvt_rs::CvtTimings::generate(1920, 1080, 60.0, cvt_rs::BlankingMode::Normal, false, false).unwrap());
    
}
