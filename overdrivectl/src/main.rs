fn main() {
    println!("Hello, world!");
    let a = cvt_rs::CvtTimings::generate(
        1280,
        1024,
        60.0,
        cvt_rs::BlankingMode::ReducedV2,
        false,
        false,
    )
    .unwrap();
let a = a.generate_modeline();
println!("{}", &a);
}