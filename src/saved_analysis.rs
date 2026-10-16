//! Advisory analysis of saved bytes. No transport, device or write entrypoints.
use crate::boards::{Analysis, BootKind, CHIP_KEY, FLASH_SIZE};
use crate::formats::{AppStatus, Package, SECTOR, app_table};
use crate::sectormap::{Region, region_ranges, sector_map};

pub const MAX_INPUT: usize = crate::formats::MAX_PACKAGE_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryState { IntactApp, CorruptApp, UnknownLayout, IncompatibleData }
impl RecoveryState {
    pub fn code(self) -> &'static str {
        match self {
            Self::IntactApp => "intact-app-checksums",
            Self::CorruptApp => "corrupt-or-incomplete-app",
            Self::UnknownLayout => "unknown-layout",
            Self::IncompatibleData => "incompatible-data-layout",
        }
    }
}

pub struct SavedAnalysis {
    pub analysis: Analysis,
    pub recovery: RecoveryState,
    pub layout_valid: bool,
    pub regions: Vec<Region>,
    // Names are used only for comparison. Never include them in a support report.
    data_layout: Vec<(String, u32, u32)>,
}


impl SavedAnalysis {
    pub fn raw(flash: &[u8]) -> Result<Self, &'static str> {
        if flash.is_empty() || flash.len() > MAX_INPUT { return Err("invalid-input-size"); }
        let analysis = Analysis::new(flash, None, CHIP_KEY);
        let valid = crate::formats::app_layout_valid(flash, CHIP_KEY);
        let has_app = app_table(flash, CHIP_KEY).iter().any(|e| e.attr == 0x82 && e.name == "app.bin");
        let recovery = match analysis.app_status {
            AppStatus::Ok if valid && has_app => RecoveryState::IntactApp,
            AppStatus::NotFound if app_table(flash, CHIP_KEY).is_empty() => RecoveryState::UnknownLayout,
            _ => RecoveryState::CorruptApp,
        };
        let regions = if valid {
            sector_map(flash, CHIP_KEY, None)
        } else {
            // Only unconditional boot/factory-address labels. No guessed stock
            // layout when an application directory cannot be trusted.
            sector_map(&vec![0xFF; flash.len()], CHIP_KEY, None)
        };
        let mut data_layout: Vec<_> = app_table(flash, CHIP_KEY).into_iter()
            .filter(|e| matches!(e.attr, 0x12 | 0x92) && e.name != "PRCT")
            .map(|e| (e.name, e.addr, e.len)).collect();
        data_layout.sort();
        Ok(Self { analysis, recovery, layout_valid: valid, regions, data_layout })
    }

    pub fn package(raw: &[u8]) -> Result<(Package, Self), &'static str> {
        if raw.len() > MAX_INPUT { return Err("invalid-input-size"); }
        let package = Package::parse(raw).map_err(|_| "invalid-package")?;
        if package.flash.len() > FLASH_SIZE { return Err("package-exceeds-historical-capacity"); }
        // Descriptors can refer to data absent from a package. Padding here is
        // an analysis convention, never evidence of acquisition of those bytes.
        let mut logical = package.flash.clone();
        logical.resize(FLASH_SIZE, 0xFF);
        let mut saved = Self::raw(&logical)?;
        if !crate::formats::app_files_present(&package.flash, CHIP_KEY) {
            saved.analysis.app_status = AppStatus::BadCrc;
            saved.recovery = RecoveryState::CorruptApp;
        }
        Ok((package, saved))
    }

    pub fn compatibility(&self, other: &Self) -> &'static str {
        if !self.layout_valid || !other.layout_valid || self.data_layout.is_empty()
            || other.data_layout.is_empty() { "unknown" }
        else if self.data_layout == other.data_layout { "matching-declarations-only" }
        else { "incompatible-data-layout" }
    }

    /// Allowlisted JSON fields only: no names, identities, evidence strings,
    /// selectors, contents, paths, receipt provenance or hashes.
    pub fn support_json(&self, size: usize, representation: &str, comparison: &str,
                        receipt: &str, integrity: &str, compatibility: &str) -> String {
        // Every string argument is supplied by a closed set in saved_cli.
        let recovery = if compatibility == "incompatible-data-layout" {
            RecoveryState::IncompatibleData
        } else { self.recovery };
        let boot = match self.analysis.bootloader {
            BootKind::Stock => "recognized-container",
            BootKind::Open => "recognized-open-container",
            BootKind::Damaged => "missing-or-corrupt",
            BootKind::Unknown => "unknown",
        };
        let ranges = Region::LEGEND.iter().filter_map(|&region| {
            let ranges = region_ranges(&self.regions, region);
            if ranges.is_empty() { return None; }
            let label = if region == Region::Free { "UNCLASSIFIED" } else { region.name() };
            Some(format!("\"{label}\":\"{ranges}\""))
        }).collect::<Vec<_>>().join(",");
        format!(concat!("{{\"schema\":\"saved-diagnostics-v1\",\"representation\":\"{}\",",
            "\"saved_bytes\":{},\"historical_capacity_bytes\":{},\"matches_historical_size\":{},",
            "\"actual_device_capacity\":\"unknown\",\"fresh_acquisition\":false,",
            "\"bootability\":\"not-demonstrated\",\"usb_route\":\"no-known-route-from-saved-bytes\",",
            "\"recovery\":\"{}\",\"boot_area\":\"{}\",\"layout_valid\":{},",
            "\"comparison\":\"{}\",\"receipt\":\"{}\",\"integrity\":\"{}\",",
            "\"data_compatibility\":\"{}\",\"sector_size\":{},\"regions\":{{{}}},",
            "\"automatic_action\":\"none\"}}"),
            representation, size, FLASH_SIZE, representation == "raw" && size == FLASH_SIZE,
            recovery.code(), boot, self.layout_valid, comparison, receipt, integrity, compatibility, SECTOR, ranges)
    }
}
