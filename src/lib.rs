#![feature(byte_slice_trim_ascii)]
use core::panic;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, BufReader, Seek, SeekFrom, BufRead};
use std::str;
use byteorder::{LittleEndian, ByteOrder};

const REQUIRED_3_1_KEYWORDS: [&str; 16] = [
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
    "$PnB", // number of bits reserved for parameter number n
    "$PnE", // amplification type for parameter n
    "$PnN", // short name for parameter n
    "$PnR", // range for parameter n
    "$TOT" // total number of events in the data set
];

const OPTIONAL_3_1_KEYWORDS: [&str; 2] = [
    "$ABRT", // events lost due to acquisition electronic coincidence
    "$BTIM"
];

pub struct Header {
    pub fcs_version: String,
    pub txt_start: u64,
    pub txt_end: u64,
    pub data_start: u64,
    pub data_end: u64,
    pub analysis_start: u64,
    pub analysis_end: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Parameter {
    name: String,
    events: Vec<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub delimitter: u8,
    pub keywords: Vec<String>,
    pub kv: HashMap<String, String>
}

/*
pub struct Analysis {
    pub gating: [i32]
}
*/

// hold flow cytometry data
pub struct FlowData {
    pub metadata: Metadata,
    //pub data: Vec<Parameter>,
    //analysis: &'a Analysis
}

/// Read FCS files
///
/// This function decodes fcs 3.0, 3.1, and 4.0 files
/// and returns the metadata and event data.
///
/// # Examples
///
/// ```
/// use flowfairy_api::read_fcs;
/// let data = read_fcs("cd8_f31.fcs");
/// let metadata = data.metadata;
/// let param_events = data.data;
/// ```
pub fn read_fcs(file_name: &str) -> Result<FlowData, io::Error> {
    let file = File::open(&file_name)?;
    let mut reader = BufReader::new(file);
    let header = read_header(&mut reader)?;
    let metadata = read_metadata(&mut reader, header.txt_start, header.txt_end)?;
    let data = read_data(&mut reader, &metadata);
    //let analysis = read_analysis(header.analysis_start, header.analysis_end);
    
    return Ok(FlowData{
        metadata,
        //data: data,
        //analysis: &analysis
    })
}

/// Read header segment of FCS file.
///
/// This function reads the FCS version, and byte offsets for text, data,
/// analysis, and supplemental text segments.
/// (read_header is used privately by the read_fcs function)
pub fn read_header(reader: &mut BufReader<File>) -> Result<Header, io::Error> {
    // read FCS version
    let mut version_buffer = [0u8; 6]; // initialize fcs version byte buffer
    reader.read_exact(&mut version_buffer)?; // read 6 bytes into version buffer
    let fcs_format = validate_fcs_version(version_buffer)?; // validate fcs version

    // read spaces
    let mut space_buffer = [0u8; 4];
    reader.read_exact(&mut space_buffer)?;
    validate_header_spaces(space_buffer)?; // confirm valid header format

    // read text, data, & analysis segment offsets
    let mut offsets = [0u64; 6];
    let mut offset_buffer = [0u8; 8];
    for i in 0..6 {
        reader.read_exact(&mut offset_buffer)?; // read byte offsets
        let trimmed_buffer = offset_buffer.trim_ascii(); // trim whitespace
        let offset_str = str::from_utf8(&trimmed_buffer)
            .expect("Unable to convert ASCII whitespace trimmed buffer to str"); // convert byte to str
        offsets[i] = offset_str.parse::<u64>() 
            .expect("Unable to convert string to u64"); // convert string bytes to u64
    }

    let header = Header{
        fcs_version: fcs_format,
        txt_start: offsets[0],
        txt_end: offsets[1],
        data_start: offsets[2],
        data_end: offsets[3],
        analysis_start: offsets[4],
        analysis_end: offsets[5],
    };

    return Ok(header)
}

/// Validate fcs version (3.0, 3.1, 4.0 supported)
///
/// This function validates the fcs version number read in the header segment.
/// Supported formats include 3.0, 3.1, 4.0.
/// (privately used by the read_header function)
fn validate_fcs_version(mut fcs_version_bytes: [u8; 6]) -> Result<String, io::Error>{
    let valid_fcs_versions = ["FCS2.0", "FCS3.0", "FCS3.1"]; // valid fcs formats
    let fcs_format = match str::from_utf8(&mut fcs_version_bytes) {
        Ok(v) => v,
        Err(e) => panic!("Error converting fcs verison to string") // add better error handling here
    };

    if valid_fcs_versions.contains(&fcs_format) {
        Ok(fcs_format.to_string())
    } else {
        panic!("Invalid FCS format") // add custom error type here
    }
}

/// Validate spaces read in the header segment of an fcs file.
/// (privately used by the read_header function)
fn validate_header_spaces(mut space_bytes: [u8; 4]) ->  Result<(), io::Error> {
    let spaces = match str::from_utf8(&mut space_bytes) {
        Ok(v) => v,
        Err(e) => panic!("Error converting spaces to string"),
    };

    if spaces != "    " {
        panic!("invalid header")// add custom error handling
    }

    Ok(())
}

/// Read metadata (keywords and values) in the text segment.
///
/// This function reads key-value pairs in the text segment of an FCS file.
/// (privately used by the read_fcs function)
pub fn read_metadata(reader: &mut BufReader<File>, start: u64, end: u64) -> Result<Metadata, io::Error> {
    let mut text_segment: Metadata = Default::default();
    reader.seek(SeekFrom::Start(start))?;

    let mut _delimitter = [0u8;1];
    reader.read_exact(&mut _delimitter)?;
    text_segment.delimitter = _delimitter[0];

    
    while reader.stream_position()? < end {
        let mut keyword: Vec<u8> = Vec::new();
        let mut value: Vec<u8> = Vec::new();
        reader.read_until(text_segment.delimitter, &mut keyword)?;
        reader.read_until(text_segment.delimitter, &mut value)?;

        let keyword_value_pair = clean_keyword_value_pair(keyword, value, text_segment.delimitter)?;
        if keyword_value_pair.0 != "" {
            text_segment.keywords.push(keyword_value_pair.0.to_owned());
            text_segment.kv.insert(keyword_value_pair.0.to_owned(), keyword_value_pair.1.to_owned());
        }
    }
    Ok(text_segment)
}

/// Validate key-value pair in the text segment
/// (used privately in the read_metadata function)
fn clean_keyword_value_pair(keyword: Vec<u8>, value: Vec<u8>, delimiter: u8) -> Result<(String, String), io::Error> {
    // check that last byte in keyword and value are delimiter character
    // FIXME: change this panic to a custom error type
    if keyword[keyword.len()-1] != delimiter || value[value.len()-1] != delimiter {
        panic!("File or parser may be corrupted")
    }
    // convert u8 vector to lossy utf8 string and remove delimiter character
    let keyword_lossy_str = str::from_utf8(&keyword[..keyword.len()-1]);
    let value_lossy_str = str::from_utf8(&value[..value.len()-1]);

    let keyword_valid_ascii = match keyword_lossy_str {
        Ok(keyword_lossy_str) => keyword_lossy_str,
        Err(_) => "",
    };

    let value_valid_ascii = match value_lossy_str {
        Ok(keyword_lossy_str) => keyword_lossy_str,
        Err(_) => "",
    };

    // trim leading and trailing whitespace
    let keyword_clean = keyword_valid_ascii.trim().to_string();
    let value_clean = value_valid_ascii.trim().to_string();

    return Ok((keyword_clean, value_clean))
}

/// Reads data segment
///
/// This function reads event data recorded in the data segment of an FCS file.
/// (privately used by the read_fcs function)
pub fn read_data(reader: &mut BufReader<File>, metadata: &Metadata) -> Result<Vec<f64>, io::Error> {
    let data_type = metadata.kv.get(&"$DATATYPE".to_string()).unwrap();

    let num_params_str = metadata.kv.get(&"$PAR".to_string()).unwrap();
    let num_events_str = metadata.kv.get(&"$TOT".to_string()).unwrap();
    let start_str = metadata.kv.get(&"$BEGINDATA".to_string()).unwrap();

    // convert strings to appropriate data types
    let num_params: usize = num_params_str.parse().unwrap();
    let num_events: usize = num_events_str.parse().unwrap();
    let start: u64 = start_str.parse().unwrap();
    let capacity = num_params * num_events;
    let byte_order = metadata.kv.get(&"BYTEORD".to_string()).unwrap();

    if capacity == 0 {
        panic!("data has len 0");
    } else {
        println!("continue");
    }

    reader.seek(SeekFrom::Start(start))?;

    let mut data: Vec<f64> = Vec::with_capacity(capacity);

    match data_type.as_str() {
        "I" => {
            for _ in 0..capacity {
                let bit_length = metadata.kv.get("P1B").unwrap();
                println!("{}", bit_length);
            }
        },
        "F" => {
            let mut buffer = [0u8; 4];
            for _ in 0..capacity {
                reader.read_exact(&mut buffer)?;
                if byte_order.as_str() == "1,2,3,4" {
                let value = f32::from_le_bytes(buffer);
                data.push(value as f64);
                } else if byte_order.as_str() == "4,3,2,1" {
                    let value = f32::from_be_bytes(buffer);
                    data.push(value as f64)
                }
            }
        },
        "D" => {
            let mut buffer = [0u8; 8];
            for _ in 0..capacity {
                reader.read_exact(&mut buffer)?;
                if byte_order.as_str() == "1,2,3,4" {
                    let value = f64::from_le_bytes(buffer);
                    data.push(value);
                } else if byte_order.as_str() == "4,3,2,1" {
                    let value = f64::from_be_bytes(buffer);
                    data.push(value);
                }
            }
        },
        "A" => {
            panic!("ASCII data type not supported in FCS3.1");
        },
        _ => panic!("Invalid data type"),
    }

    
    return Ok(data)
}

