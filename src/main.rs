use xml::reader::{EventReader, XmlEvent as ReaderEvent};
use xml::writer::{EmitterConfig, EventWriter, XmlEvent as WriterEvent};
use xml::name::Name;
use hex::encode;
use std::fs::{File};
use walkdir::{DirEntry, WalkDir};
use std::fs;
use std::str::from_utf8;
use std::path::Path;
use md5;
use std::env;
use std::io::BufReader;

fn main() {
    let text = " <root>
    <xsd:schema id=\"root\" xmlns=\"\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:msdata=\"urn:schemas-microsoft-com:xml-msdata\">\r\n    <xsd:import namespace=\"http://www.w3.org/XML/1998/namespace\" /> <xsd:element name=\"root\" msdata:IsDataSet=\"true\">\r\n      <xsd:complexType>\r\n        <xsd:choice maxOccurs=\"unbounded\">\r\n          <xsd:element name=\"metadata\">\r\n            <xsd:complexType>\r\n              <xsd:sequence>\r\n                <xsd:element name=\"value\" type=\"xsd:string\" minOccurs=\"0\" />\r\n              </xsd:sequence>\r\n              <xsd:attribute name=\"name\" use=\"required\" type=\"xsd:string\" />\r\n              <xsd:attribute name=\"type\" type=\"xsd:string\" />\r\n              <xsd:attribute name=\"mimetype\" type=\"xsd:string\" />\r\n              <xsd:attribute ref=\"xml:space\" />\r\n            </xsd:complexType>\r\n          </xsd:element>\r\n          <xsd:element name=\"assembly\">\r\n            <xsd:complexType>\r\n              <xsd:attribute name=\"alias\" type=\"xsd:string\" />\r\n              <xsd:attribute name=\"name\" type=\"xsd:string\" />\r\n            </xsd:complexType>\r\n          </xsd:element>\r\n          <xsd:element name=\"data\">\r\n            <xsd:complexType>\r\n              <xsd:sequence>\r\n                <xsd:element name=\"value\" type=\"xsd:string\" minOccurs=\"0\" msdata:Ordinal=\"1\" />\r\n                <xsd:element name=\"comment\" type=\"xsd:string\" minOccurs=\"0\" msdata:Ordinal=\"2\" />\r\n              </xsd:sequence>\r\n              <xsd:attribute name=\"name\" type=\"xsd:string\" use=\"required\" msdata:Ordinal=\"1\" />\r\n              <xsd:attribute name=\"type\" type=\"xsd:string\" msdata:Ordinal=\"3\" />\r\n              <xsd:attribute name=\"mimetype\" type=\"xsd:string\" msdata:Ordinal=\"4\" />\r\n              <xsd:attribute ref=\"xml:space\" />\r\n            </xsd:complexType>\r\n          </xsd:element>\r\n          <xsd:element name=\"resheader\"><xsd:complexType><xsd:sequence> <xsd:element name=\"value\" type=\"xsd:string\" minOccurs=\"0\" msdata:Ordinal=\"1\" />  </xsd:sequence>          <xsd:attribute name=\"name\" type=\"xsd:string\" use=\"required\" /></xsd:complexType></xsd:element>     </xsd:choice>   </xsd:complexType> </xsd:element>
    </xsd:schema>
    <data name=\"ProcessInfo123\" xml:space=\"preserve\">
      <value>Process information:</value>
      <comment>md5:fa690fed8c8ffde6eec7d4fe7cd5f2ba</comment>
    </data>
    <data name=\"ProcessInfo123\" xml:space=\"preserve\">
      <value>Process inasdasdsstion:</value>
      <comment>md5:fa690fed8c8ffde6eec7d4fe7cd5f2ba</comment>
    </data>
</root>";

    let args : Vec<String> = env::args().collect();
    let path = &args[1];

    let walker = WalkDir::new(path).into_iter();
    let entrys = walker.filter_entry(|e| (e.file_type().is_file() && e.path().extension().unwrap() == "resx") || e.file_name() != "hashed");


    for entry in entrys {
        let entry = entry.unwrap();

        if entry.file_type().is_dir(){
            continue;
        }

        println!("{:?}", entry.file_name());
        let file = File::open(entry.path()).unwrap();
        let file = BufReader::new(file);
        let reader = EventReader::new(file);

        let mut new_file_path = entry.clone().into_path();
        new_file_path.pop();
        new_file_path.push("hashed");
        fs::create_dir_all(&new_file_path).unwrap();

        new_file_path.push(entry.file_name());
        println!("{:?}", new_file_path);
        let mut file = File::create(new_file_path).unwrap();
        let mut writer = EmitterConfig::new().perform_indent(true).create_writer(&mut file);
        let mut in_data =false;
        let mut in_comment = false;
        let mut in_value =false;
        let mut localization_value: String = "".to_string();
        let empty_string = "".to_string();

        for e in reader {
            match e {
                Ok(ReaderEvent::StartElement {name, attributes, namespace}) if name.local_name == "data" => {
                    in_data = true;
                    let mut start_data = WriterEvent::start_element(name.borrow());
                    
                    for attr in &attributes {
                        let attr_name = Name{
                            local_name: attr.name.local_name.as_str(),
                            namespace: attr.name.namespace_ref(),
                            prefix: attr.name.prefix_ref(),
                        };

                        start_data = start_data.attr(attr_name, attr.value.as_str());
                    }
    
                    writer.write(start_data).unwrap();
                    println!("Start data");
                }
                Ok(ReaderEvent::StartElement {name, attributes, namespace}) if name.local_name == "value" => {
                    if in_data{
                        in_value = true;
                    }
                    let mut start_value = WriterEvent::start_element(name.borrow());
                    for attr in &attributes {
                        start_value = start_value.attr(name.local_name.as_str(), attr.value.as_str());
                    }
    
                    writer.write(start_value).unwrap();
    
                    println!("Start value");
                }
                Ok(ReaderEvent::StartElement {name, attributes, namespace}) if name.local_name == "comment" => {
                  if in_data {
                      in_comment = true;
                  }
    
                  let mut start_comment = WriterEvent::start_element(name.borrow());
                    for attr in &attributes {
                        start_comment = start_comment.attr(name.local_name.as_str(), attr.value.as_str());
                    }
    
                    writer.write(start_comment).unwrap();
    
                    println!("Start comment");
                }
                Ok(ReaderEvent::Characters(text)) => {
                    
                    println!("{:?}", text);
                    if in_comment {
                        let digest = md5::compute(&localization_value);
                        let value = digest.0;
                        let value_as_string = encode(&value);
                        let mut md5 = "md5:".to_string();
                        md5.push_str(value_as_string.as_str());
    
                        writer.write(WriterEvent::characters(&md5)).unwrap();
                    }
                    else if in_value {
                        localization_value = text.clone();
                        in_value = false;
                        writer.write(WriterEvent::characters(text.as_str())).unwrap();
                    }
                    else{
                        writer.write(WriterEvent::characters(text.as_str())).unwrap();
                    }
    
                }
    
                Ok(ReaderEvent::EndElement {name}) if name.local_name == "data" => {
                    if !in_comment {
                        let comment_start = WriterEvent::start_element("comment");
                        writer.write(comment_start).unwrap();
    
                        let digest = md5::compute(&localization_value);
                        let value = digest.0;
                        let value_as_string = hex::encode(&value);
                        let mut md5 = "md5:".to_string();
                        md5.push_str(value_as_string.as_str());
                        let comment_content = WriterEvent::characters(&md5);
                        writer.write(comment_content).unwrap();
    
                        let comment_end = WriterEvent::end_element();
                        writer.write(comment_end).unwrap();
                        writer.write(WriterEvent::end_element()).unwrap();
                        println!("End_Data");
                    }
                    else{
                        in_comment = false;
                        in_data = false;
                        writer.write(WriterEvent::end_element()).unwrap();
                        println!("End_{:?}", name.local_name);
                    }
                }
                Ok(event) => {
                    if let Some(e) = event.as_writer_event() {
                        match writer.write(e) {
                            Err(error) => {
                                println!("{:?}", error);
                                println!("{:?}", event);
                                panic!("1234");
                            }
                            Ok(_) => {
                                println!("{:?}", event);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("{:?}", e);
                    println!("Some Error");
                }
            }
        }
    }

    
    

    // reader.trim_text(true);
    // let mut writer = Writer::new(Cursor::new(Vec::new()));
    // let mut localization_value: &[u8] = b"asdf";
    // let mut in_data = false;
    // let mut in_value = false;
    // let mut in_comment = false;
    // let mut buffer = Vec::new();
    // loop{
    //     match reader.read_event(&mut buffer){
    //         Ok(Event::Start(ref e)) if e.name() == b"value" => {
    //             in_value = true;
    //             writer.write_event(
    //                 Event::Start(
    //                     BytesStart::borrowed(e.name().clone(), e.name().len())
    //                 )
    //             ).expect("Error adding event to writer");
    //         },
    //         Ok(Event::Start(ref e)) if e.name() == b"data" => {
    //             in_data = true;
    //             writer.write_event(
    //                 Event::Start(
    //                     BytesStart::borrowed(e.name().clone(), e.name().len())
    //                 )
    //             ).expect("Error adding event to writer");
    //         },
    //         Ok(Event::Start(ref e)) if e.name() == b"comment" => {
    //             if in_data {
    //                 in_comment = true
    //             }
    //             writer.write_event(
    //                 Event::Start(
    //                     BytesStart::borrowed(e.name().clone(), e.name().len())
    //                 )
    //             ).expect("Error adding event to writer");
    //         },
    //         Ok(Event::Text(_)) if in_comment => {
    //             let hash = md5::compute(localization_value);
    //             let bytes_text = BytesText::from_plain(&hash.0);
    //             writer.write_event(Event::Text(bytes_text)).expect("Error adding event to writer");

    //         },
    //         Ok(Event::Text(ref e)) if in_value && in_data => {
    //             in_value = false;
    //             localization_value = e.clone().escaped();
    //             writer.write_event(
    //                 Event::Text(
    //                     BytesText::from_escaped(e.escaped())
    //                 )
    //             ).expect("Error adding event to writer");
    //         },
    //         Ok(Event::End(ref e)) if e.name() == b"data" => {
    //             if !in_comment {
    //                 writer.write_event(
    //                     Event::Start(
    //                         BytesStart::owned(b"comment".to_vec(), "comment".len())
    //                     )
    //                 ).expect("Error adding event to writer");

    //             }
    //             in_comment = false;
    //             in_data = false;
    //             writer.write_event(
    //                 Event::End(
    //                     BytesEnd::borrowed(e.name().clone())
    //                 )
    //             ).expect("Error adding event to writer");
    //         },
    //         Ok(Event::Eof) => break,
    //         Ok(e) => {
    //             writer.write_event(e).expect("Error adding event to writer");
    //         },
    //         Err(_) => {
    //             panic!("Error parsing xml file");
    //         },
    //     }
    //}
}