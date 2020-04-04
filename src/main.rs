use xml::reader::{EventReader, XmlEvent as ReaderEvent};
use xml::writer::{EmitterConfig, XmlEvent as WriterEvent, EventWriter};
use xml::name::{Name, OwnedName};
use xml::attribute::OwnedAttribute;
use xml::namespace::Namespace;
use hex::encode;
use std::fs::File;
use walkdir::{WalkDir, DirEntry};
use std::fs;
use md5;
use std::env;
use std::io::BufReader;
use std::path::{Path, PathBuf};

fn main() {
    let args : Vec<String> = env::args().collect();
    let path = &args[1];

    let walker = WalkDir::new(path).into_iter();
    let entrys = walker.filter_entry(|e| (e.file_type().is_file() && e.path().extension().unwrap() == "resx") || e.file_name() != "hashed");


    for entry in entrys {
        let entry = entry.unwrap();

        if entry.file_type().is_dir(){
            continue;
        }

        if cfg!(debug_assertions) {
            println!("{:?}", entry.file_name());
        }

        let reader = create_reader(entry.path());

        let mut file = create_hashed_file(entry);
        let mut writer = EmitterConfig::new().perform_indent(true).create_writer(&mut file);

        let mut in_data = false;
        let mut in_comment = false;
        let mut in_value = false;
        let mut localization_value: String = "".to_string();

        for e in reader {
            match e {
                Ok(ReaderEvent::StartElement {name, attributes, namespace: _}) if name.local_name == "data" => {
                    in_data = true;
                    
                    copy_and_write_start_element(name, attributes, &mut writer);

                    if cfg!(debug_assertions) {
                        println!("Start data");
                    }
                }
                Ok(ReaderEvent::StartElement {name, attributes, namespace: _}) if name.local_name == "value" => {
                    if in_data{
                        in_value = true;
                    }

                    copy_and_write_start_element(name, attributes, &mut writer);
    
                    if cfg!(debug_assertions) {
                        println!("Start value");
                    }
                }
                Ok(ReaderEvent::StartElement {name, attributes, namespace: _}) if name.local_name == "comment" => {
                    if in_data {
                        in_comment = true;
                    }
    
                    copy_and_write_start_element(name, attributes, &mut writer);
                    
                    if cfg!(debug_assertions) {
                        println!("Start comment");
                    }
                }
                Ok(ReaderEvent::Characters(text)) => {
                    
                    if cfg!(debug_assertions) {
                        println!("{:?}", text);
                    }
                    if in_comment {
                        create_and_write_md5_string(&localization_value, &mut writer);
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
    
                        create_and_write_md5_string(&localization_value, &mut writer);
    
                        let comment_end = WriterEvent::end_element();
                        writer.write(comment_end).unwrap();
                        writer.write(WriterEvent::end_element()).unwrap();
                        
                        if cfg!(debug_assertions) {
                            println!("End_Data");
                        }
                    }
                    else{
                        in_comment = false;
                        in_data = false;
                        writer.write(WriterEvent::end_element()).unwrap();
                        
                        if cfg!(debug_assertions) {
                            println!("End_{:?}", name.local_name);
                        }
                    }
                }
                Ok(event) => {
                    if let Some(e) = event.as_writer_event() {
                        match writer.write(e) {
                            Err(error) => {
                                println!("{:?}", error);
                                println!("{:?}", event);
                            }
                            Ok(_) => {
                                if cfg!(debug_assertions) {
                                    println!("{:?}", event);
                                }
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

    fn create_reader(path: &Path) -> EventReader<BufReader<File>> {
        let file = File::open(path).unwrap();
        let file = BufReader::new(file);
        EventReader::new(file)
    }

    fn create_hashed_file(entry: DirEntry) -> File {
        let mut new_file_path = entry.clone().into_path();
        new_file_path.pop();
        new_file_path.push("hashed");
        fs::create_dir_all(&new_file_path).unwrap();

        new_file_path.push(entry.file_name());
        if cfg!(debug_assertions) {
            println!("{:?}", new_file_path);
        }
        File::create(new_file_path).unwrap()
    }

    fn copy_and_write_start_element(name: OwnedName, attributes: Vec<OwnedAttribute>, writer: &mut EventWriter<&mut File>){
        let element_name = Name {
            local_name: name.local_name.as_str(),
            namespace: name.namespace_ref(),
            prefix: name.prefix_ref(),
        };
        
        let mut start_element = WriterEvent::start_element(element_name);
                    
        for attr in &attributes {
            let attr_name = Name{
                local_name: attr.name.local_name.as_str(),
                namespace: attr.name.namespace_ref(),
                prefix: attr.name.prefix_ref(),
            };

            start_element = start_element.attr(attr_name, attr.value.as_str());
        }

        writer.write(start_element).unwrap();
    }

    fn create_and_write_md5_string(localization_value: &String, writer: &mut EventWriter<&mut File>) {
        let md5_string = create_md5_string(localization_value);
        writer.write(WriterEvent::characters(&md5_string)).unwrap();
    }

    fn create_md5_string(localization_value: &String) -> String {
        let digest = md5::compute(&localization_value);
        let value = digest.0;
        let value_as_string = encode(&value);
        let mut md5 = "md5:".to_string();
        md5.push_str(value_as_string.as_str());
        md5
    }
}