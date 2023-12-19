use std::{
    fs,
    io::{BufReader, Read, Seek, Write},
    path::{Path, PathBuf},
    vec,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
struct Package {
    content: Content,
}

#[derive(Debug, Deserialize, Serialize)]
struct Content {
    resources: Vec<Resources>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Resources {
    target: String,
    resource: Vec<Resource>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Resource {
    r#type: String,
    subdir: String,
    includes: Vec<Includes>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Includes {
    pattern: String,
}

fn extract_svd_paths_from_compressed_manifest(manifest: &mut impl Read) -> Result<Vec<String>> {
    let reader = BufReader::new(manifest);
    let package: Package = quick_xml::de::from_reader(reader)?;

    let mut svds_paths = Vec::<String>::new();
    package.content.resources.iter().for_each(|e| {
        e.resource.iter().for_each(|r| {
            if r.r#type == "svd" {
                if let Some(i) = r.includes.first() {
                    svds_paths.push([r.subdir.clone(), i.pattern.clone()].join("/"));
                }
            }
        })
    });

    Ok(svds_paths)
}

pub fn extract_svds_from_pack(
    atpack: &mut (impl Read + Seek),
    destination: &Path,
) -> Result<Vec<String>> {
    let mut archive = ZipArchive::new(atpack)?;
    let mut manifest = archive.by_name("package.content")?;

    let svds_paths = extract_svd_paths_from_compressed_manifest(&mut manifest)?;

    drop(manifest);

    let mut successful_svds: Vec<String> = vec![];

    for svd_path in svds_paths {
        let mut svd = archive.by_name(&svd_path)?;
        let mut content = String::with_capacity(1000000);
        svd.read_to_string(&mut content)?;

        let svd_path = PathBuf::from(svd_path);
        let filename = svd_path.file_name().unwrap(); // TODO: to error if not present
        fs::create_dir_all(destination)?;
        let path = destination.join(&filename);

        let mut file = fs::File::create(path)?; // TODO: extract file name only
        file.write_all(&content.as_bytes())?;

        successful_svds.push(filename.to_string_lossy().to_string());
    }

    Ok(successful_svds)
}

#[cfg(test)]
mod test {
    use std::{
        ffi::OsStr,
        fs::{self, File},
        io::Error as IoError,
        path::{Path, PathBuf},
    };

    use super::{Content, Includes, Package, Resource, Resources};
    use indoc::indoc;
    use tempdir::TempDir;

    #[test]
    fn try_serialize() {
        let package = Package {
            content: Content {
                resources: vec![Resources {
                    target: "ATSAMV71J19B".to_owned(),
                    resource: vec![Resource {
                        r#type: "svd".to_owned(),
                        subdir: "samv71b/svd".to_owned(),
                        includes: vec![Includes {
                            pattern: "ATSAMV71J19B.svd".to_owned(),
                        }],
                    }],
                }],
            },
        };

        let xml = quick_xml::se::to_string(&package).unwrap();

        println!("XML: {}", xml);
    }

    #[test]
    fn check_content() {
        static PACKAGE_CONTENT_MANIFEST: &str = indoc!(
            r#"
            <?xml version='1.0' encoding='ASCII'?>
            <package schemaVersion="1.0">
            <content>
                <resources target="ATSAMV71J19B">
                    <resource type="atdf" subdir="samv71b/atdf">
                        <includes pattern="ATSAMV71J19B.atdf"/>
                    </resource>
                    <resource type="pic" subdir="samv71b/edc">
                        <includes pattern="ATSAMV71J19B.PIC"/>
                    </resource>
                    <resource type="svd" subdir="samv71b/svd">
                        <includes pattern="ATSAMV71J19B.svd"/>
                    </resource>
                    <resource type="c.header" subdir="samv71b/include">
                        <includes pattern="sam.h"/>
                        <meta key="define" value="__SAMV71J19B__"/>
                    </resource>
                    <resource type="c.source.exe.template" subdir="samv71b/templates">
                        <includes pattern="main.c"/>
                    </resource>
                    <resource type="c.source.lib.template" subdir="samv71b/templates">
                        <includes pattern="library.c"/>
                    </resource>
                    <resource type="cpp.source.exe.template" subdir="samv71b/templates">
                        <includes pattern="main.cpp"/>
                    </resource>
                    <resource type="cpp.source.lib.template" subdir="samv71b/templates">
                        <includes pattern="library.cpp"/>
                    </resource>
                    <resource type="armcc.source.startup" subdir="samv71b/armcc">
                        <includes pattern="armcc/startup_samv71j19b.s"/>
                        <includes pattern="system_samv71j19b.c"/>
                    </resource>
                   <resource type="keil.flashloader" subdir="samv71b/keil">
                        <includes pattern="flash/ATSAMV7x_512.FLM"/>
                        <includes pattern="flash/ATSAMV7x_GPNVM.FLM"/>
                        <includes pattern="debug/SAMx7.dbgconf"/>
                        <meta key="flash/ATSAMV7x_512.FLM" value="start=0x00400000|size=0x00080000|default=1"/>
                        <meta key="flash/ATSAMV7x_GPNVM.FLM" value="start=0x1FFFFFF0|size=0x00000010|default=0"/>
                    </resource>
                </resources>
                <resources target="ATSAMV71J20B">
                    <resource type="atdf" subdir="samv71b/atdf">
                        <includes pattern="ATSAMV71J20B.atdf"/>
                    </resource>
                    <resource type="pic" subdir="samv71b/edc">
                        <includes pattern="ATSAMV71J20B.PIC"/>
                    </resource>
                    <resource type="svd" subdir="samv71b/svd">
                        <includes pattern="ATSAMV71J20B.svd"/>
                    </resource>
                    <resource type="c.header" subdir="samv71b/include">
                        <includes pattern="sam.h"/>
                    </resource>
                </resources>
            </content>
            </package>
        "#
        );
        let package: Package =
            quick_xml::de::from_str(&PACKAGE_CONTENT_MANIFEST).expect("Shall deserialize");

        let mut found = false;
        package.content.resources.iter().for_each(|e| {
            if e.target == "ATSAMV71J20B" {
                e.resource.iter().for_each(|r| {
                    if r.r#type == "svd" {
                        if let Some(i) = r.includes.first() {
                            found = i.pattern == "ATSAMV71J20B.svd";
                        }
                    }
                })
            }
        });

        assert!(found);
    }

    #[test]
    fn check_svd_paths_extraction() {
        let mut f = File::open("test/data/package.content").expect("Test file not opened");
        let svds_paths =
            super::extract_svd_paths_from_compressed_manifest(&mut f).expect("Extraction failed");

        println!("{:?}", svds_paths);

        assert!(svds_paths
            .iter()
            .any(|e| e == "samv71b/svd/ATSAMV71J19B.svd"));
        assert!(svds_paths
            .iter()
            .any(|e| e == "samv71b/svd/ATSAMV71J20B.svd"));
        assert!(svds_paths
            .iter()
            .any(|e| e == "samv71b/svd/ATSAMV71J21B.svd"));
        assert!(svds_paths
            .iter()
            .any(|e| e == "samv71b/svd/ATSAMV71N19B.svd"));
        assert!(svds_paths
            .iter()
            .any(|e| e == "samv71b/svd/ATSAMV71N20B.svd"));
    }

    #[test]
    fn check_svd_extraction() {
        let tempdir = TempDir::new("atpack-svds").expect("Temporary directory creation failed");
        let mut archive = File::open("test/data/test.atpack").expect("Test archive not opened");
        let _ =
            super::extract_svds_from_pack(&mut archive, tempdir.path()).expect("Extraction failed");

        //let paths = fs::read_dir(tempdir.path()).expect("Failed to read temporary directory");

        let entries = fs::read_dir(tempdir.path())
            .expect("Failed to read temporary directory")
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, IoError>>()
            .expect("Failed to collect");

        println!("* Entries: {:?}", entries);

        assert!(entries
            .iter()
            .any(|f| f.file_name() == Some(OsStr::new("ATSAMV71J19B.svd"))));
        assert!(entries
            .iter()
            .any(|f| f.file_name() == Some(OsStr::new("ATSAMV71J20B.svd"))));
        assert!(entries
            .iter()
            .any(|f| f.file_name() == Some(OsStr::new("ATSAMV71J21B.svd"))));
        assert!(entries
            .iter()
            .any(|f| f.file_name() == Some(OsStr::new("ATSAMV71N19B.svd"))));
        assert!(entries
            .iter()
            .any(|f| f.file_name() == Some(OsStr::new("ATSAMV71N20B.svd"))));
    }
}
