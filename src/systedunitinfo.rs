use zbus::dbus_proxy;

use crate::Message;
use iced::theme::Container;
use iced::widget::{container, row, text};
use iced::{Alignment, Element, Length};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::sync::OnceLock;

static SESSION: OnceLock<zbus::Connection> = OnceLock::new();

#[derive(Debug, thiserror::Error, Clone, Copy)]
pub enum UnitGetError {
    #[error("Error during zbus tokio thread")]
    ZbusThreadError,
    #[error("Xml is broken")]
    XmlError,
}

async fn get_connection() -> zbus::Result<zbus::Connection> {
    if let Some(cnx) = SESSION.get() {
        Ok(cnx.clone())
    } else {
        let cnx = zbus::Connection::session().await?;
        SESSION.set(cnx.clone()).expect("Can't reset a OnceCell");
        Ok(cnx)
    }
}

fn names_from_xml(xml: String) -> Result<Vec<String>, quick_xml::Error> {
    let mut interfaces = Vec::new();
    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);
    reader.expand_empty_elements(true);
    let mut buf = Vec::new();
    loop {
        let event = reader.read_event_into(&mut buf)?;
        match event {
            Event::Start(element) => {
                if let b"node" = element.name().as_ref() {
                    for att in element.attributes().flatten() {
                        if att.key.as_ref() == b"name" {
                            interfaces
                                .push(att.decode_and_unescape_value(&mut reader)?.to_string());
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(interfaces)
}

#[derive(Debug, Clone)]
pub struct UnitInfo {
    originunit: String,
    can_freeze: bool,
    collect_mode: String,
    id: String,
}

impl UnitInfo {
    pub fn view(&self) -> Element<Message> {
        let row: Element<Message> = row![
            text(self.originunit.as_str()).width(Length::Fixed(350_f32)),
            text(self.can_freeze.to_string()).width(Length::Fixed(60_f32)),
            text(self.collect_mode.to_string()).width(Length::Fixed(60_f32)),
            text(self.id.to_string()),
        ]
        .spacing(10)
        .align_items(Alignment::Start)
        .into();

        container(row)
            .width(Length::Fill)
            .style(Container::Box)
            .padding(10)
            .into()
    }
}

#[derive(Debug, Clone)]
pub struct UnitInterfaceInfoVec(Vec<UnitInfo>);

impl UnitInterfaceInfoVec {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &UnitInfo> {
        self.0.iter()
    }

    pub async fn refresh(&self) -> Result<Self, UnitGetError> {
        let conn = get_connection()
            .await
            .map_err(|_| UnitGetError::ZbusThreadError)?;
        let systembus = SystemdDbusProxy::new(&conn)
            .await
            .map_err(|_| UnitGetError::ZbusThreadError)?;
        let xml = systembus
            .introspect()
            .await
            .map_err(|_| UnitGetError::ZbusThreadError)?;
        let mut unitvec = Vec::new();
        let names = names_from_xml(xml).map_err(|_| UnitGetError::XmlError)?;
        for unit in names {
            let unitbus = Systemd1UnitProxy::builder(&conn)
                .path(format!("/org/freedesktop/systemd1/unit/{unit}"))
                .map_err(|_| UnitGetError::ZbusThreadError)?
                .build()
                .await
                .map_err(|_| UnitGetError::ZbusThreadError)?;
            unitvec.push(UnitInfo {
                originunit: unit,
                can_freeze: unitbus
                    .can_freeze()
                    .await
                    .map_err(|_| UnitGetError::ZbusThreadError)?,
                collect_mode: unitbus
                    .collect_mode()
                    .await
                    .map_err(|_| UnitGetError::ZbusThreadError)?,
                id: unitbus
                    .id()
                    .await
                    .map_err(|_| UnitGetError::ZbusThreadError)?,
            });
        }
        Ok(Self(unitvec))
    }
}

#[dbus_proxy(
    default_service = "org.freedesktop.systemd1",
    interface = "org.freedesktop.DBus.Introspectable",
    default_path = "/org/freedesktop/systemd1/unit"
)]
trait SystemdDbus {
    fn introspect(&self) -> zbus::Result<String>;
}
#[dbus_proxy(
    interface = "org.freedesktop.systemd1.Unit",
    default_service = "org.freedesktop.systemd1"
)]
trait Systemd1Unit {
    #[dbus_proxy(property)]
    fn can_freeze(&self) -> zbus::Result<bool>;
    #[dbus_proxy(property)]
    fn collect_mode(&self) -> zbus::Result<String>;
    #[dbus_proxy(property)]
    fn id(&self) -> zbus::Result<String>;
}
