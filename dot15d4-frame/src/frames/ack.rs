use crate::{Error, Result};

use crate::{
    AddressingFields, AddressingMode, AuxiliarySecurityHeader, FrameControl, FrameType,
    FrameVersion, InformationElements,
};

pub struct Ack<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> Ack<T> {
    pub fn new(buffer: T) -> Result<Self> {
        let ack = Self::new_unchecked(buffer);

        if !ack.check_len() {
            return Err(Error);
        }

        Ok(ack)
    }

    pub fn check_len(&self) -> bool {
        let buffer = self.buffer.as_ref();

        if buffer.len() != 3 {
            return false;
        }

        true
    }

    pub fn new_unchecked(buffer: T) -> Self {
        Self { buffer }
    }

    pub fn frame_control(&self) -> FrameControl<&'_ [u8]> {
        FrameControl::new(&self.buffer.as_ref()[..2]).unwrap()
    }

    pub fn sequence_number(&self) -> u8 {
        self.buffer.as_ref()[2]
    }
}

pub struct EnhancedAck<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> EnhancedAck<T> {
    pub fn new(buffer: T) -> Result<Self> {
        let ack = Self::new_unchecked(buffer);

        //if !ack.check_len() {
        //return Err(Error);
        //}

        Ok(ack)
    }

    pub fn check_len(&self) -> bool {
        todo!();
    }

    pub fn new_unchecked(buffer: T) -> Self {
        Self { buffer }
    }

    pub fn frame_control(&self) -> FrameControl<&'_ [u8]> {
        FrameControl::new(&self.buffer.as_ref()[..2]).unwrap()
    }

    pub fn sequence_number(&self) -> Option<u8> {
        if self.frame_control().sequence_number_suppression() {
            None
        } else {
            Some(self.buffer.as_ref()[2])
        }
    }

    pub fn addressing(&self) -> Option<AddressingFields<&'_ [u8], &'_ [u8]>> {
        if self.frame_control().sequence_number_suppression() {
            AddressingFields::new(&self.buffer.as_ref()[2..], self.frame_control()).ok()
        } else {
            AddressingFields::new(&self.buffer.as_ref()[3..], self.frame_control()).ok()
        }
    }

    pub fn auxiliary_security_header(&self) -> Option<AuxiliarySecurityHeader<&'_ [u8]>> {
        let mut offset = 2;

        if !self.frame_control().sequence_number_suppression() {
            offset += 1;
        }

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        if self.frame_control().security_enabled() {
            Some(AuxiliarySecurityHeader::new(
                &self.buffer.as_ref()[offset..],
            ))
        } else {
            None
        }
    }

    pub fn information_elements(&self) -> Option<InformationElements<&'_ [u8]>> {
        let mut offset = 2;

        if !self.frame_control().sequence_number_suppression() {
            offset += 1;
        }

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        if self.frame_control().security_enabled() {
            if let Some(ash) = self.auxiliary_security_header() {
                offset += ash.len();
            }
        }

        if self.frame_control().information_elements_present() {
            Some(InformationElements::new(&self.buffer.as_ref()[offset..]).unwrap())
        } else {
            None
        }
    }
}

impl<T: AsRef<[u8]> + ?Sized> EnhancedAck<&T> {
    pub fn payload(&self) -> Option<&'_ [u8]> {
        let fc = self.frame_control();

        let mut offset = 2;

        if !fc.sequence_number_suppression() {
            offset += 1;
        }

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        if fc.security_enabled() {
            if let Some(ash) = self.auxiliary_security_header() {
                offset += ash.len();
            }
        }

        if fc.information_elements_present() {
            if let Some(ie) = self.information_elements() {
                offset += ie.len();
            }
        }

        Some(&self.buffer.as_ref()[offset..])
    }
}
