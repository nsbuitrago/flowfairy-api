#![feature(byte_slice_trim_ascii)]
use core::panic;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, SeekFrom, Seek, BufRead};
use std::str;
use byteorder::{ReadBytesExt, LittleEndian, BigEndian};
use regex::RegexSet;

const REQUIRED_KEYWORDS: [&str; 12] = [
    "$BEGINANALYSIS", // byte-offset to the beginning of analysis segment
    "$BEGINDATA", // byte-offset of beginning of data segment
    "$BEGINSTEXT", // byte-offset to beginning of text segment
    "$BYTEORD", // byte order for data acquisition computer
    "$DATATYPE", // type of data in data segment (ASCII, int, float)
    "$ENDANALYSIS", // byte-offset to end of analysis segment
    "$ENDDATA", // byte-offset to end of data segment
    "$ENDSTEXT", // byte-offset to end of text segment
    "$MODE", // data mode (list mode - preferred, histogram - deprecated)
    "$NEXTDATA", // byte-offset to next data set in the file
    "$PAR", // number of parameters in an event
    "$TOT" // total number of events in the data set
];

const OPTIONAL_KEYWORDS: [&str; 31] = [
    "$ABRT", // events lost due to acquisition electronic coincidence
    "$BTIM", // clock time at beginning of data acquisition
    "$CELLS", // description of objects measured
    "$COM", // comment
    "$CSMODE", // cell subset mode, number of subsets an object may belong
    "$CSVBITS", // number of bits used to encode cell subset identifier
    "$CYT", // cytometer type
    "$CYTSN", // cytometer serial number
    "$DATE", // date of data acquisition
    "$ETIM", // clock time at end of data acquisition
    "$EXP", // investigator name initiating experiment
    "$FIL", // name of data file containing data set
    "$GATE", // number of gating parameters
    "$GATING", // region combinations used for gating
    "$INST", // institution where data was acquired
    "$LAST_MODIFIED", // timestamp of last modification
    "$LAST_MODIFIER", // person performing last modification
    "$LOST", // number events lost due to computer busy
    "$OP", // name of flow cytometry operator
    "$ORIGINALITY", // information whether FCS data set has been modified or not
    "$PLATEID", // plate identifier
    "$PLATENAME", // plate name
    "$PROJ", // project name
    "$SMNO", // specimen (i.e., tube) label
    "$SPILLOVER", // spillover matrix
    "$SRC", // source of specimen (cell type, name, etc.)
    "$SYS", // type of computer and OS
    "$TIMESTEP", // time step for time parameter
    "$TR", // trigger paramter and its threshold
    "$VOL", // volume of sample run during data acquisition
    "$WELLID" // well identifier
];

/// FlowData struct containing metadata and parameter event data read from an FCS file.
pub struct FlowData {
    pub metadata: Metadata,
    pub data: Vec<Parameter>
}

/// Metadata containing the FCS file version carried over from the Header struct, 
/// delimitter for the text segment, keywords, and values from the text segment of an FCS file.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub version: String,
    pub delimitter: u8,
    pub keywords: Vec<String>,
    pub values: HashMap<String, String>
}

/// Parameter struct containing the parameter id (name) and its corresponding event data.
pub struct Parameter {
    pub id: String,
    pub events: Vec<f64>
}

/// Header struct containing the FCS file version and byte offsets to data segements in an FCS file.
pub struct Header {
    pub version: String,
    pub txt_start: u64,
    pub txt_end: u64,
    pub data_start: u64,
    pub data_end: u64,
    pub analysis_start: u64,
    pub analysis_end: u64
}

/// Read FCS files
///
/// This function reads fcs files and returns a FlowData struct containing
/// metadata as well as parameter event data.
pub fn read_fcs(filename: &str) -> Result<FlowData, io::Error> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let metadata = read_metadata(&mut reader)?;
    let data = read_data(&mut reader, &metadata)?; // read data segment

    let flowdata = FlowData{
        metadata: metadata,
        data: data
    };

    return Ok(flowdata)
}

/// Read header segment of an fcs file
fn read_header(reader: &mut BufReader<File>) -> Result<Header, io::Error> {
    let mut buffer = [0u8; 8]; 

    reader.read_exact(&mut buffer[..6])?;
    let fcs_version = validate_fcs_version(&buffer[..6])?;

    reader.read_exact(&mut buffer[..4])?;
    validate_spaces(&buffer[..4])?;

    let mut offsets = [0u64; 6];
    for i in 0..6 {
        reader.read_exact(&mut buffer)?;
        let trimmed_buffer = buffer.trim_ascii();
        let byte_offset = str::from_utf8(&trimmed_buffer)
            .expect("Unablel to convert byte array to str");
        offsets[i] = byte_offset.parse::<u64>()
            .expect("Unable to convert str to u64");
    }

    let header = Header{
        version: fcs_version,
        txt_start: offsets[0],
        txt_end: offsets[1],
        data_start: offsets[2],
        data_end: offsets[3],
        analysis_start: offsets[4],
        analysis_end: offsets[5]
    };

    return Ok(header)
}

// Check that read FCS version is supported
fn validate_fcs_version(mut bytes: &[u8]) -> Result<String, io::Error>{
    let valid_versions = ["FCS3.0", "FCS3.1"];
    let fcs_version = str::from_utf8(&mut bytes)
        .expect("Could not convert bytes to string");

    if valid_versions.contains(&fcs_version) {
        return Ok(fcs_version.to_string()) 
    } else {
        panic!("Warning, FCS version {} not supported", fcs_version)
    }
}

// Check that the correct spacing is found in between the FCS version and byte offsets in the text segment
fn validate_spaces(mut bytes: &[u8]) -> Result<String, io::Error> {
    let spaces = str::from_utf8(&mut bytes)
        .expect("Could not convert bytes to string");

    if spaces == "    " {
        return Ok(spaces.to_string())
    } else {
        panic!("Invalid number of spaces")
    }
}

/// Reads text segment of an fcs file
/// FIXME: Currently does not support keywords or values escaped by delimitter
fn read_metadata(reader: &mut BufReader<File>) -> Result<Metadata, io::Error> {
    let header = read_header(reader)?;

    let mut metadata = Metadata::default();
    metadata.version = header.version;
    reader.seek(SeekFrom::Start(header.txt_start))?;

    let delimitter = reader.read_u8()?;
    metadata.delimitter = delimitter;

    while reader.stream_position()? < header.txt_end {
        let mut keyword: Vec<u8> = Vec::new();
        let mut value: Vec<u8> = Vec::new();
        reader.read_until(delimitter, &mut keyword)?;
        reader.read_until(delimitter, &mut value)?;

        let (keyword, value) = clean_kv(&keyword, &value);

        if keyword != "" {
            metadata.keywords.push(keyword.to_owned());
            metadata.values.insert(keyword, value);
        }
    }
    validate_metadata(&metadata);
    return Ok(metadata)
}

// Convert keyword and value byte arrays to strings, trim whitespace, and remove delimitter
fn clean_kv(keyword: &Vec<u8>, value: &Vec<u8>) -> (String, String) {
    let keyword = str::from_utf8(&keyword[..keyword.len()-1]);
    let value = str::from_utf8(&value[..value.len()-1]);

    let keyword = match keyword {
        Ok(keyword) => keyword.trim(),
        Err(_) => ""
    };

    let value = match value {
        Ok(value) => value.trim(),
        Err(_) => ""
    };
    return (keyword.to_string(), value.to_string())
}

// Validate that all read keywords are valid and that all required keywords are present
fn validate_metadata(metadata: &Metadata) {

    // check that all required keywords are present
    for keyword in REQUIRED_KEYWORDS.iter() {
        // also check parameter specific required keywords
        if !metadata.keywords.contains(&keyword.to_string()) {
            panic!("Required keyword {} is missing", keyword);
        }
    }

    let total_params = metadata.values.get("$PAR").unwrap();
    let n_digits = total_params.chars().count().to_string();
    let regex_string = r"[PR]\d{1,".to_string() + &n_digits + "}[BENRDFGLOPSTVIW]";
    let param_keywords = RegexSet::new(&[regex_string,]).unwrap();

    // check that all keywords are valid
    for keyword in metadata.keywords.iter() {
        if !REQUIRED_KEYWORDS.contains(&keyword.as_str()) && !OPTIONAL_KEYWORDS.contains(&keyword.as_str()) && !param_keywords.is_match(&keyword.as_str()) {
            panic!("Keyword {} is not a valid keyword", keyword);
        }
    }
}

/// Read data segment from an fcs file
fn read_data(reader: &mut BufReader<File>, metadata: &Metadata) -> Result<Vec<Parameter>, io::Error> {
    let data_mode: &str = metadata.values.get("$MODE").unwrap();
    // FIXME: add error handling here
    if data_mode != "L" {
        panic!("Data mode {} not supported", data_mode);
    }

    let data_type: &str = metadata.values.get("$DATATYPE").unwrap().as_str();
    let total_params: usize = metadata.values.get("$PAR").unwrap().parse().unwrap();
    let total_events: usize = metadata.values.get("$TOT").unwrap().parse().unwrap();
    let start_offset: u64 = metadata.values.get("$BEGINDATA").unwrap().parse().unwrap();
    //let end_offset: u64 = metadata.values.get("$ENDDATA").unwrap().parse().unwrap();
    let byte_order: &str = metadata.values.get("$BYTEORD").unwrap().as_str();
    let capacity: usize = total_params * total_events;

    if capacity == 0 {
        panic!("No data in file");
    }

    reader.seek(SeekFrom::Start(start_offset))?;
    let mut data: Vec<f64> = Vec::with_capacity(capacity);

    match data_type {
        "I" => {
            // 
            for _ in 0..capacity {
                let value = reader.read_i32::<LittleEndian>()?;
                data.push(value as f64);
            }
        },
        "F" => {
            for _ in 0..capacity {
                if byte_order == "1,2,3,4" {
                    let value = reader.read_f32::<LittleEndian>()?;
                    data.push(value as f64);
                } else if byte_order == "4,3,2,1" {
                    let value = reader.read_f32::<BigEndian>()?;
                    data.push(value as f64)
                }
            }
        },
        "D" => {
            for _ in 0..capacity {
                if byte_order == "1,2,3,4,5,6,7,8" {
                    let value = reader.read_f64::<LittleEndian>()?;
                    data.push(value);
                } else if byte_order == "8,7,6,5,4,3,2,1" {
                    let value = reader.read_f64::<BigEndian>()?;
                    data.push(value);
                }
            }
        },
        _ => panic!("Invalid data type")
    }

    // once we have data, let's assign events to a parameter
    // get all parameter names in order (P1N, P2N, etc)
    let mut parameter_events: Vec<Parameter> = Vec::new();
    for i in 0..total_params {
        let param_keyword = format!("$P{}N", i+1);
        let id = metadata.values.get(&param_keyword).unwrap().to_owned();
        let mut events: Vec<f64> = Vec::new();

        for j in 0..total_events {
            let index = i * total_events + j;
            events.push(data[index]);
        }

        let param = Parameter{
            id,
            events
        };
        parameter_events.push(param);
    }

    return Ok(parameter_events)
}





