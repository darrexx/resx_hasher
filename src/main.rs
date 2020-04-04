use xml::reader::{EventReader, XmlEvent as ReaderEvent};
use xml::writer::{EmitterConfig, XmlEvent as WriterEvent, EventWriter};
use xml::name::{Name, OwnedName};
use xml::attribute::OwnedAttribute;
use hex::encode;
use std::fs::File;
use walkdir::{WalkDir, DirEntry};
use std::fs;
use md5;
use std::env;
use std::io::{Read, BufReader, Write};
use std::path::Path;

fn main() {
    let args : Vec<String> = env::args().collect();
    let path = &args[1];

    let walker = WalkDir::new(path).into_iter();
    let entrys = walker.filter_entry(|e|
           (e.file_type().is_file() && e.path().extension().unwrap() == "resx")
        || (e.file_type().is_dir() && e.file_name() != "hashed")
    );

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
        let writer = EmitterConfig::new().perform_indent(true).create_writer(&mut file);

        Hasher::hash(reader, writer);
    }
}

struct Hasher<W> {
    writer: EventWriter<W>,
    in_data: bool,
    in_comment: bool,
    in_value: bool,
    localization_value: String,
}

impl<W: Write> Hasher<W> {
    fn hash<R: Read>(reader: EventReader<R>, writer: EventWriter<W>) {
        let mut this = Hasher {
            writer,
            in_data: false,
            in_comment: false,
            in_value: false,
            localization_value: String::new(),
        };

        for e in reader {
            match e {
                Ok(ReaderEvent::StartElement {name, attributes, ..}) if name.local_name == "data" =>
                    this.on_data_start(name, attributes),
                Ok(ReaderEvent::StartElement {name, attributes, ..}) if name.local_name == "value" =>
                    this.on_value_start(name, attributes),
                Ok(ReaderEvent::StartElement {name, attributes, ..}) if name.local_name == "comment" =>
                    this.on_comment(name, attributes),
                Ok(ReaderEvent::Characters(text)) =>
                    this.on_characters(text),
                Ok(ReaderEvent::EndElement {name}) if name.local_name == "data" =>
                    this.on_data_end(name),
                Ok(event) =>
                    this.on_other_event(event),
                Err(e) => {
                    println!("{:?}", e);
                    println!("Some Error");
                }
            }
        }
    }

    fn on_data_start(&mut self, name: OwnedName, attributes: Vec<OwnedAttribute>) {
        self.in_data = true;
                    
        self.copy_and_write_start_element(name, attributes);

        if cfg!(debug_assertions) {
            println!("Start data");
        }
    }

    fn on_value_start(&mut self, name: OwnedName, attributes: Vec<OwnedAttribute>) {
        if self.in_data {
            self.in_value = true;
        }

        self.copy_and_write_start_element(name, attributes);

        if cfg!(debug_assertions) {
            println!("Start value");
        }    
    }

    fn on_comment(&mut self, name: OwnedName, attributes: Vec<OwnedAttribute>) {
        if self.in_data {
            self.in_comment = true;
        }

        self.copy_and_write_start_element(name, attributes);
        
        if cfg!(debug_assertions) {
            println!("Start comment");
        }
    }

    fn on_characters(&mut self, text: String) {
        if cfg!(debug_assertions) {
            println!("{:?}", text);
        }

        if self.in_comment {
            create_and_write_md5_string(&self.localization_value, &mut self.writer);
        }
        else if self.in_value {
            self.localization_value = text.clone();
            self.in_value = false;
            self.writer.write(WriterEvent::characters(text.as_str())).unwrap();
        }
        else{
            self.writer.write(WriterEvent::characters(text.as_str())).unwrap();
        }
    }

    fn on_data_end(&mut self, name: OwnedName) {
        if !self.in_comment {
            let comment_start = WriterEvent::start_element("comment");
            self.writer.write(comment_start).unwrap();

            create_and_write_md5_string(&self.localization_value, &mut self.writer);

            let comment_end = WriterEvent::end_element();
            self.writer.write(comment_end).unwrap();
            self.writer.write(WriterEvent::end_element()).unwrap();
            
            if cfg!(debug_assertions) {
                println!("End_Data");
            }
        }
        else{
            self.in_comment = false;
            self.in_data = false;
            self.writer.write(WriterEvent::end_element()).unwrap();
            
            if cfg!(debug_assertions) {
                println!("End_{:?}", name.local_name);
            }
        }
    }

    fn on_other_event(&mut self, event: ReaderEvent) {
        if let Some(e) = event.as_writer_event() {
            match self.writer.write(e) {
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

    fn copy_and_write_start_element(&mut self, name: OwnedName, attributes: Vec<OwnedAttribute>){
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

        self.writer.write(start_element).unwrap();
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

fn create_and_write_md5_string<W: Write>(localization_value: &str, writer: &mut EventWriter<W>) {
    let md5_string = create_md5_string(localization_value);
    writer.write(WriterEvent::characters(&md5_string)).unwrap();
}

fn create_md5_string(localization_value: &str) -> String {
    let digest = md5::compute(&localization_value);
    let value = digest.0;
    let value_as_string = encode(&value);
    let mut md5 = "md5:".to_string();
    md5.push_str(value_as_string.as_str());
    md5
}
