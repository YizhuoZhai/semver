use crate::{Comparator, Op, Version, VersionReq};

pub(crate) fn matches_req(req: &VersionReq, ver: &Version) -> bool {
    for cmp in &req.comparators {
        if !matches_impl(cmp, ver) {
            return false;
        }
    }

    if ver.pre.is_empty() {
        return true;
    }

    // If a version has a prerelease tag (for example, 1.2.3-alpha.3) then it
    // will only be allowed to satisfy req if at least one comparator with the
    // same major.minor.patch also has a prerelease tag.
    for cmp in &req.comparators {
        if pre_is_compatible(cmp, ver) {
            return true;
        }
    }

    false
}

pub(crate) fn matches_comparator(cmp: &Comparator, ver: &Version) -> bool {
    matches_impl(cmp, ver) && (ver.pre.is_empty() || pre_is_compatible(cmp, ver))
}

fn matches_impl(cmp: &Comparator, ver: &Version) -> bool {
    match cmp.op {
        Op::Exact | Op::Wildcard => matches_exact(cmp, ver),
        Op::Greater => matches_greater(cmp, ver),
        Op::GreaterEq => matches_exact(cmp, ver) || matches_greater(cmp, ver),
        Op::Less => matches_less(cmp, ver),
        Op::LessEq => matches_exact(cmp, ver) || matches_less(cmp, ver),
        Op::Tilde => matches_tilde(cmp, ver),
        Op::Caret => matches_caret(cmp, ver),
        #[cfg(no_non_exhaustive)]
        Op::__NonExhaustive => unreachable!(),
    }
}

fn matches_exact(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    if let Some(minor) = cmp.minor {
        if ver.minor != minor {
            return false;
        }
    }

    if let Some(patch) = cmp.patch {
        if ver.patch != patch {
            return false;
        }
    }

    ver.pre == cmp.pre
}

fn matches_greater(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return ver.major > cmp.major;
    }

    match cmp.minor {
        None => return false,
        Some(minor) => {
            if ver.minor != minor {
                return ver.minor > minor;
            }
        }
    }

    match cmp.patch {
        None => return false,
        Some(patch) => {
            if ver.patch != patch {
                return ver.patch > patch;
            }
        }
    }

    ver.pre > cmp.pre
}

fn matches_less(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return ver.major < cmp.major;
    }

    match cmp.minor {
        None => return false,
        Some(minor) => {
            if ver.minor != minor {
                return ver.minor < minor;
            }
        }
    }

    match cmp.patch {
        None => return false,
        Some(patch) => {
            if ver.patch != patch {
                return ver.patch < patch;
            }
        }
    }

    ver.pre < cmp.pre
}

fn matches_tilde(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    if let Some(minor) = cmp.minor {
        if ver.minor != minor {
            return false;
        }
    }

    if let Some(patch) = cmp.patch {
        if ver.patch != patch {
            return ver.patch > patch;
        }
    }

    ver.pre >= cmp.pre
}

fn matches_caret(cmp: &Comparator, ver: &Version) -> bool {
    if ver.major != cmp.major {
        return false;
    }

    let minor = match cmp.minor {
        None => return true,
        Some(minor) => minor,
    };

    let patch = match cmp.patch {
        None => {
            if cmp.major > 0 {
                return ver.minor >= minor;
            } else {
                return ver.minor == minor;
            }
        }
        Some(patch) => patch,
    };

    if cmp.major > 0 {
        if ver.minor != minor {
            return ver.minor > minor;
        } else if ver.patch != patch {
            return ver.patch > patch;
        }
    } else if minor > 0 {
        if ver.minor != minor {
            return false;
        } else if ver.patch != patch {
            return ver.patch > patch;
        }
    } else if ver.minor != minor || ver.patch != patch {
        return false;
    }

    ver.pre >= cmp.pre
}

fn pre_is_compatible(cmp: &Comparator, ver: &Version) -> bool {
    cmp.major == ver.major
        && cmp.minor == Some(ver.minor)
        && cmp.patch == Some(ver.patch)
        && !cmp.pre.is_empty()
}

#[cfg(test)]
mod test_matches {
    use super::*;
    use crate::{Comparator, Op, Prerelease, Version, BuildMetadata};
    #[test]
    fn test_matches_caret_major() {
        let major_version = 1;

        let major_1  = 1;
        let minor_1 = 0;
        let patch_1  = 0;
        //let rug_fuzz_2 = 0;
        
        let major_2 = 2;
        let minor_2 = 0;
        let patch_2 = 0;
        let cmp = Comparator {
            op: Op::Caret,
            major: major_version,
            minor: None,
            patch: None,
            pre: Prerelease::EMPTY,
        };
        let ver = Version::new(major_1, minor_1, patch_1);
        assert!(matches_caret(& cmp, & ver));
        let ver = Version::new(major_2, minor_2, patch_2);
        assert!(! matches_caret(& cmp, & ver));

    }
    #[test]
    fn test_matches_caret_minor() {
        let major_version = 1;
        let minor_version = 2;

        let major_1 = 1;
        let minor_1 = 2;
        let patch_1 = 0;

        let major_2 = 2;
        let minor_2 = 0;
        let patch_2 = 0;

        let major_3 = 1;
        let minor_3 = 3;
        let patch_3 = 0;

        let cmp = Comparator {
            op: Op::Caret,
            major: major_version,
            minor: Some(minor_version),
            patch: None,
            pre: Prerelease::EMPTY,
        };
        let ver = Version::new(major_1, minor_1, patch_1);
        assert!(matches_caret(& cmp, & ver));
        let ver = Version::new(major_2, minor_2, patch_2);
        assert!(! matches_caret(& cmp, & ver));
        let ver = Version::new(major_3, minor_3, patch_3);
        assert!(matches_caret(& cmp, & ver));
    }
    #[test]
    fn test_matches_caret_patch() {
        let major_0 = 0;
        let minor_0 = 2;
        let patch_0 = 3;

        let major_1 = 0;
        let minor_1 = 2;
        let patch_1 = 3;

        let major_2 = 0;
        let minor_2 = 3;
        let patch_2 = 0;

        let major_3 = 0;
        let minor_3 = 2;
        let patch_3 = 4;
        let cmp = Comparator {
            op: Op::Caret,
            major: major_0,
            minor: Some(minor_0),
            patch: Some(patch_0),
            pre: Prerelease::EMPTY,
        };
        let ver = Version::new(major_1, minor_1, patch_1);
        assert!(matches_caret(& cmp, & ver));
        let ver = Version::new(major_2, minor_2, patch_2);
        assert!(! matches_caret(& cmp, & ver));
        let ver = Version::new(major_3, minor_3, patch_3);
        assert!(matches_caret(& cmp, & ver));
    }
    #[test]
    fn test_matches_caret_build() {
        let major_version = 1;

        let major_1 = 1;
        let minor_1 = 0;
        let patch_1 = 0;
        let build_text_0 = "build";

        let major_2 = 2;
        let minor_2 = 0;
        let patch_2 = 0;

        let build_text_1 = "build";
        let cmp = Comparator {
            op: Op::Caret,
            major: major_version,
            minor: None,
            patch: None,
            pre: Prerelease::EMPTY,
        };
        let ver = Version {
            major: major_1,
            minor: minor_1,
            patch: patch_1,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::new(build_text_0).unwrap(),
        };
        assert!(matches_caret(& cmp, & ver));
        let ver = Version {
            major: major_2,
            minor: minor_2,
            patch: patch_2,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::new(build_text_1).unwrap(),
        };
        assert!(! matches_caret(& cmp, & ver));
    }
    #[test]
    fn test_matches_caret_zero_major() {
        let major_version = 0;

        let major_1 = 0;
        let minor_1 = 1;
        let patch_1 = 0;

        let major_2 = 1;
        let minor_2 = 0;
        let patch_2 = 0;

        let major_3 = 0;
        let minor_3 = 0;
        let patch_3 = 1;
        let cmp = Comparator {
            op: Op::Caret,
            major: major_version,
            minor: None,
            patch: None,
            pre: Prerelease::EMPTY,
        };
        let ver = Version::new(major_1, minor_1, patch_1);
        assert!(matches_caret(& cmp, & ver));
        let ver = Version::new(major_2, minor_2, patch_2);
        assert!(! matches_caret(& cmp, & ver));
        let ver = Version::new(major_3, minor_3, patch_3);
        assert!(matches_caret(& cmp, & ver));
    }
}