
/// Represents CVT timings.
/// To better understand CVT timings, read the README of this crate
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CvtTimings {
    /// The pixel clock (MHz)
    pub pixel_clock: f64,

    /// Total width to scan (Pixels)
    pub h_total: u32,
    /// Active width section to scan (Pixels)
    pub h_active: u32,
    /// Blanking width section to scan (Pixels)
    pub h_blank: u32,
    /// Front Porch width section to scan (Pixels)
    pub h_front_porch: u32,
    /// Horizontal Sync (Pixels)
    pub h_sync: u32,
    /// Back Porch width section to scan (Pixels)
    pub h_back_porch: u32,
    /// Polarity of the horizontal sync scan (+/-)
    pub h_sync_polarity: bool,
    /// Horizontal scan frequency (KHz)
    /// This represents how many times per second a horizontal scan is performed
    pub h_freq: f64,
    /// Horizontal scan period (us)
    /// This represents the amount of time a horizontal scan takes
    pub h_period: f64,

    /// Total height to scan (Pixels)
    pub v_total: u32,
    /// Active height section to scan (Pixels)
    pub v_active: u32,
    /// Blanking height section to scan (Pixels)
    pub v_blank: u32,
    /// Front Porch height section to scan (Pixels)
    pub v_front_porch: u32,
    /// Vertical Sync (Pixels)
    pub v_sync: u32,
    /// Back Porch height section to scan (Pixels)
    pub v_back_porch: u32,
    /// Polarity of the vertical sync scan (+/-)
    pub v_sync_polarity: bool,
    /// Vertical scan frequency (KHz)
    /// This represents how many times per second a vertical scan is performed
    pub v_freq: f64,
    /// Vertical scan period (us)
    /// This represents the amount of time a vertical scan takes
    pub v_period: f64,

    /// Whether the video stream is interlaced or not
    pub interlaced: bool,
}

/// Monitor blanking mode
/// Determines the way a monitor should draw frames to the screen and the amount of bandwidth they use on the wire.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BlankingMode {
    /// Normal blanking.
    /// does not cut down any time, works on all monitor types.
    /// If you want to overdrive a non-CRT monitor, you should check out [BlankingMode::ReducedV2] or [BlankingMode::Reduced].
    Normal,
    /// Reduced blanking.
    /// Does not work on CRT displays, but cuts down significantly on the bandwidth required.
    /// You should use [BlankingMode::ReducedV2] instead, it being more efficient, unless it causes issues with your monitor.
    Reduced,
    /// Reduced blanking V2.
    /// Does not work on CRT displays, but cuts down significantly on the bandwidth required, even more than [BlankingMode::Reduced].
    /// If it causes issues with your monitor, switch to [BlankingMode::Reduced] or [BlankingMode::Normal].
    ReducedV2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum AspectRatio {
    Aspect4by3,
    Aspect16by9,
    Aspect16by10,
    Aspect5by4,
    Aspect15by9,

    /// NOTE: NOT DEFINED BY THE SPEC
    Aspect43by18,
    Aspect64by27,
    Aspect12by5,
    AspectUnknown,
}

impl CvtTimings {
    // generates  video timings according to the VESA CVT standard
    // look into GTF for the future
    // https://glenwing.github.io/docs/VESA-GTF-1.1.pdf

    /// Generates CVT timings according to the input given.
    pub fn generate(
        h_pixels: u32,
        v_pixels: u32,
        refresh_rate: f64,
        blanking_mode: BlankingMode,
        margins: bool,
        interlaced: bool,
    ) -> Self {
        let clock_step: f64;
        let min_v_bporch: u32 = 6;
        let rb_h_blank: u32;
        let _rb_h_sync: u32 = 32;
        let rb_min_v_blank: u32 = 460;
        let rb_v_fporch: u32;
        let refresh_multiplier: f64;
        let h_pol: bool;
        let v_pol: bool;
        let cell_gran: f64 = 8.0;
        let margin_per: f64 = 1.8;
        let min_vsync_bp: f64 = 550.0;
        let min_v_porch_rnd: f64 = 3.0;
        let c_prime: u32 = 30;
        let m_prime: u32 = 300;
        let h_sync_per: f64 = 0.08;

        match blanking_mode {
            BlankingMode::Normal => {
                clock_step = 0.25;
                rb_h_blank = 160;
                rb_v_fporch = 3;
                refresh_multiplier = 1.0;
                h_pol = false;
                v_pol = true;
            }
            BlankingMode::Reduced => {
                clock_step = 0.25;
                rb_h_blank = 160;
                rb_v_fporch = 3;
                refresh_multiplier = 1.0;
                h_pol = true;
                v_pol = false;
            }
            BlankingMode::ReducedV2 => {
                clock_step = 0.001;
                rb_h_blank = 80;
                rb_v_fporch = 1;
                refresh_multiplier = 1.0; // video optimized shit idk
                h_pol = true;
                v_pol = false;
            }
        }

        let cell_gran_rnd = cell_gran.floor();

        // 5.2 Computation of Common Parameters
        // if interlacing, set it to 2x the refresh rate
        let v_field_rate_rqd = if interlaced {
            refresh_rate * 2.0
        } else {
            refresh_rate
        };

        let h_pixels_rnd = (h_pixels as f64 / cell_gran_rnd).floor() * cell_gran_rnd;

        // if using margins, set the margin
        let left_margin = if margins {
            ((h_pixels_rnd * margin_per / 100.0) / cell_gran_rnd).floor() * cell_gran_rnd
        } else {
            0.0
        };
        let right_margin = left_margin;

        let total_active_pixels = (h_pixels_rnd + left_margin + right_margin).floor() as u32; // floor?

        let v_lines_rnd = if interlaced {
            ((v_pixels as f64) / 2.0).floor()
        } else {
            (v_pixels as f64).floor()
        };

        let top_margin = if margins {
            (v_lines_rnd * margin_per / 100.0).floor()
        } else {
            0.0
        };
        let bot_margin = top_margin;

        let interlace = if interlaced { 0.5 } else { 0.0 };

        let v_sync_rnd: f64;

        if blanking_mode == BlankingMode::ReducedV2 {
            v_sync_rnd = 8.0;
        } else {
            // calculate aspect ratio
            let aspect_ratio =
                get_aspect_ratio(interlaced, v_lines_rnd, h_pixels_rnd, cell_gran_rnd);

            match aspect_ratio {
                AspectRatio::Aspect4by3 => {
                    v_sync_rnd = 4.0;
                }
                AspectRatio::Aspect16by9 => {
                    v_sync_rnd = 5.0;
                }
                AspectRatio::Aspect16by10 => {
                    v_sync_rnd = 6.0;
                }
                AspectRatio::Aspect5by4 => {
                    v_sync_rnd = 7.0;
                }
                AspectRatio::Aspect15by9 => {
                    v_sync_rnd = 7.0;
                }
                _ => {
                    v_sync_rnd = 10.0;
                }
            }
        }

        let h_period_est: f64;
        let mut v_sync_bp: f64;
        let v_blank: f64;
        let v_front_porch: f64;
        let v_back_porch: f64;
        let total_v_lines: f64;
        let ideal_duty_cycle: f64;
        let h_blank: f64;
        let total_pixels: f64;
        let h_sync: f64;
        let h_back_porch: f64;
        let h_front_porch: f64;
        let act_pix_freq: f64;
        if blanking_mode == BlankingMode::Normal {
            h_period_est = ((1.0 / v_field_rate_rqd) - min_vsync_bp / 1000000.0)
                / (v_lines_rnd + (2.0 * top_margin) + min_v_porch_rnd + interlace)
                * 1000000.0;
            v_sync_bp = (min_vsync_bp / h_period_est).floor() + 1.0;

            if v_sync_bp < (v_sync_rnd + min_v_bporch as f64) {
                v_sync_bp = v_sync_rnd + min_v_bporch as f64;
            }
            v_blank = v_sync_bp + min_v_porch_rnd;
            v_front_porch = min_v_porch_rnd;
            v_back_porch = v_sync_bp - v_sync_rnd;
            total_v_lines =
                v_lines_rnd + top_margin + bot_margin + v_sync_bp + interlace + min_v_porch_rnd;
            ideal_duty_cycle = c_prime as f64 - (m_prime as f64 * h_period_est / 1000.0);

            if ideal_duty_cycle < 20.0 {
                h_blank =
                    (total_active_pixels as f64 * 20.0 / (100.0 - 20.0) / (2.0 * cell_gran_rnd))
                        * (2.0 * cell_gran_rnd);
            } else {
                h_blank = (total_active_pixels as f64 * ideal_duty_cycle
                    / (100.0 - ideal_duty_cycle)
                    / (2.0 * cell_gran_rnd))
                    .floor()
                    * (2.0 * cell_gran_rnd);
            }
            total_pixels = total_active_pixels as f64 + h_blank;

            h_sync = (h_sync_per * total_pixels / cell_gran_rnd).floor() * cell_gran_rnd;
            h_back_porch = h_blank / 2.0;
            h_front_porch = h_blank - h_sync - h_back_porch;
            act_pix_freq = clock_step * (total_pixels / h_period_est / clock_step).floor();
        } else {
            h_period_est = ((1000000.0 / v_field_rate_rqd) - rb_min_v_blank as f64)
                / (v_lines_rnd + top_margin + bot_margin);
            h_blank = rb_h_blank as f64;
            let vbi_lines = (rb_min_v_blank as f64 / h_period_est) + 1.0;
            let rb_min_vbi = rb_v_fporch as f64 + v_sync_rnd + min_v_bporch as f64;
            let act_vbi_lines = if vbi_lines < rb_min_vbi {
                rb_min_vbi
            } else {
                vbi_lines
            };
            total_v_lines = act_vbi_lines + v_lines_rnd + top_margin + bot_margin + interlace;
            total_pixels = (rb_h_blank + total_active_pixels) as f64;
            act_pix_freq = clock_step
                * ((v_field_rate_rqd * total_v_lines * total_pixels / 1000000.0
                    * refresh_multiplier)
                    / clock_step)
                    .floor();

            if blanking_mode == BlankingMode::ReducedV2 {
                v_blank = act_vbi_lines;
                v_front_porch = act_vbi_lines - v_sync_rnd - 6.0;
                v_back_porch = 6.0;
                h_sync = 32.0;
                h_back_porch = 40.0;
                h_front_porch = h_blank - h_sync - h_back_porch;
            } else {
                v_blank = act_vbi_lines;
                v_front_porch = 3.0;
                v_back_porch = act_vbi_lines - v_front_porch - v_sync_rnd;

                h_sync = 32.0;
                h_back_porch = 80.0;
                h_front_porch = h_blank - h_sync - h_back_porch;
            }
        }

        let pclock = act_pix_freq * 1000000.0;
        let h_freq = pclock / total_pixels;
        let v_freq = pclock / (total_v_lines * total_pixels);
        Self {
            pixel_clock: pclock,
            h_active: total_active_pixels,
            h_blank: h_blank as u32,
            h_total: total_pixels as u32,
            v_active: v_lines_rnd as u32,
            v_blank: v_blank as u32,
            v_total: total_v_lines as u32,
            h_freq: (h_freq * 100.0).round() / 100.0,
            v_freq: (v_freq * 100.0).round() / 100.0,
            h_period: 1.0 / h_freq,
            v_period: 1.0 / v_freq,
            h_front_porch: h_front_porch as u32,
            h_sync: h_sync as u32,
            h_back_porch: h_back_porch as u32,
            h_sync_polarity: h_pol,
            v_front_porch: v_front_porch as u32,
            v_sync: v_sync_rnd as u32,
            v_back_porch: v_back_porch as u32,
            v_sync_polarity: v_pol,
            interlaced,
        }
    }

    pub fn generate_modeline(&self) -> String {
        format!(
            "Modeline \"{}x{}_{:.2}{}\" {} {} {} {} {} {} {} {} {} {} {} {}",
            self.h_active,
            self.v_active,
            self.v_freq,
            if self.interlaced { "i" } else { "" },
            (self.pixel_clock / 1000.0).round() / 1000.0,
            self.h_active,
            self.h_active + self.h_front_porch,
            self.h_active + self.h_front_porch + self.h_sync,
            self.h_total,
            self.v_active,
            self.v_active + self.v_front_porch,
            self.v_active + self.v_front_porch + self.v_sync,
            self.v_total,
            if self.h_sync_polarity {
                "+HSync"
            } else {
                "-HSync"
            },
            if self.v_sync_polarity {
                "+Vsync"
            } else {
                "-VSync"
            },
            if self.interlaced { "Interlace" } else { "" }
        )
    }
}

fn get_aspect_ratio(
    interlaced: bool,
    v_lines_rnd: f64,
    h_pixels_rnd: f64,
    cell_gran_rnd: f64,
) -> AspectRatio {
    let ver_pixels = if interlaced {
        2.0 * v_lines_rnd
    } else {
        v_lines_rnd
    };
    let hor_pixels_4_3 = cell_gran_rnd * (ver_pixels * 4.0 / 3.0).floor() / cell_gran_rnd;
    let hor_pixels_16_9 = cell_gran_rnd * (ver_pixels * 16.0 / 9.0).floor() / cell_gran_rnd;
    let hor_pixels_16_10 = cell_gran_rnd * (ver_pixels * 16.0 / 10.0).floor() / cell_gran_rnd;
    let hor_pixels_5_4 = cell_gran_rnd * (ver_pixels * 5.0 / 4.0).floor() / cell_gran_rnd;
    let hor_pixels_15_9 = cell_gran_rnd * (ver_pixels * 15.0 / 9.0).floor() / cell_gran_rnd;
    let hor_pixels_43_18 = cell_gran_rnd * (ver_pixels * 43.0 / 18.0).floor() / cell_gran_rnd;
    let hor_pixels_64_27 = cell_gran_rnd * (ver_pixels * 64.0 / 27.0).floor() / cell_gran_rnd;
    let hor_pixels_12_5 = cell_gran_rnd * (ver_pixels * 12.0 / 5.0).floor() / cell_gran_rnd;

    if hor_pixels_4_3 == h_pixels_rnd {
        AspectRatio::Aspect4by3
    } else if hor_pixels_16_9 == h_pixels_rnd {
        return AspectRatio::Aspect16by9;
    } else if hor_pixels_16_10 == h_pixels_rnd {
        return AspectRatio::Aspect16by10;
    } else if hor_pixels_5_4 == h_pixels_rnd {
        return AspectRatio::Aspect5by4;
    } else if hor_pixels_15_9 == h_pixels_rnd {
        return AspectRatio::Aspect15by9;
    } else if hor_pixels_43_18 == h_pixels_rnd {
        return AspectRatio::Aspect43by18;
    } else if hor_pixels_64_27 == h_pixels_rnd {
        return AspectRatio::Aspect64by27;
    } else if hor_pixels_12_5 == h_pixels_rnd {
        return AspectRatio::Aspect12by5;
    } else {
        return AspectRatio::AspectUnknown;
    }
}