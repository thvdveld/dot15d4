use crate::{Error, Result};

use crate::{
    AddressingFields, AddressingMode, AuxiliarySecurityHeader, FrameControl, FrameType,
    FrameVersion, InformationElements,
};
use crate::{AddressingFieldsRepr, FrameControlRepr, InformationElementsRepr};

/// A reader/writer for an IEEE 802.15.4 Data frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataFrame<T: AsRef<[u8]>> {
    buffer: T,
}

impl<T: AsRef<[u8]>> DataFrame<T> {
    /// Create a new [`Frame`] reader/writer from a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is too short to contain a valid frame.
    pub fn new(buffer: T) -> Result<Self> {
        let b = Self::new_unchecked(buffer);

        if !b.check_len() {
            return Err(Error);
        }

        let fc = b.frame_control();

        if fc.security_enabled() {
            return Err(Error);
        }

        if fc.frame_type() == FrameType::Unknown {
            return Err(Error);
        }

        if fc.frame_version() == FrameVersion::Unknown {
            return Err(Error);
        }

        if fc.dst_addressing_mode() == AddressingMode::Unknown {
            return Err(Error);
        }

        if fc.src_addressing_mode() == AddressingMode::Unknown {
            return Err(Error);
        }

        Ok(b)
    }

    /// Returns `false` if the buffer is too short to contain a valid frame.
    fn check_len(&self) -> bool {
        let buffer = self.buffer.as_ref();

        if buffer.len() < 2 || buffer.len() > 127 {
            return false;
        }

        let fc = self.frame_control();

        if !fc.sequence_number_suppression() && buffer.len() < 3 {
            return false;
        }

        true
    }

    /// Create a new [`Frame`] reader/writer from a given buffer without length
    /// checking.
    pub fn new_unchecked(buffer: T) -> Self {
        Self { buffer }
    }

    /// Return a [`FrameControl`] reader.
    pub fn frame_control(&self) -> FrameControl<&'_ [u8]> {
        FrameControl::new_unchecked(&self.buffer.as_ref()[..2])
    }

    /// Return the sequence number if not suppressed.
    pub fn sequence_number(&self) -> Option<u8> {
        if self.frame_control().sequence_number_suppression() {
            None
        } else {
            Some(self.buffer.as_ref()[2])
        }
    }

    /// Return an [`AddressingFields`] reader.
    pub fn addressing(&self) -> Option<AddressingFields<&'_ [u8], &'_ [u8]>> {
        let fc = self.frame_control();

        if fc.sequence_number_suppression() {
            AddressingFields::new(&self.buffer.as_ref()[2..], fc).ok()
        } else {
            AddressingFields::new(&self.buffer.as_ref()[3..], fc).ok()
        }
    }

    /// Return an [`AuxiliarySecurityHeader`] reader.
    pub fn auxiliary_security_header(&self) -> Option<AuxiliarySecurityHeader<&'_ [u8]>> {
        let fc = self.frame_control();

        if fc.security_enabled() {
            let mut offset = 2;

            offset += !fc.sequence_number_suppression() as usize;

            if let Some(af) = self.addressing() {
                offset += af.len();
            }

            Some(AuxiliarySecurityHeader::new(
                &self.buffer.as_ref()[offset..],
            ))
        } else {
            None
        }
    }

    /// Return an [`InformationElements`] reader.
    pub fn information_elements(&self) -> Option<InformationElements<&'_ [u8]>> {
        let fc = self.frame_control();
        if fc.information_elements_present() {
            let mut offset = 2;
            offset += !fc.sequence_number_suppression() as usize;

            if let Some(af) = self.addressing() {
                offset += af.len();
            }

            Some(InformationElements::new(&self.buffer.as_ref()[offset..]).ok()?)
        } else {
            None
        }
    }
}

impl<'f, T: AsRef<[u8]> + ?Sized> DataFrame<&'f T> {
    /// Return the payload of the frame.
    pub fn payload(&self) -> Option<&'f [u8]> {
        let fc = self.frame_control();

        let mut offset = 0;
        offset += 2;

        if !fc.sequence_number_suppression() {
            offset += 1;
        }

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        if fc.security_enabled() {
            offset += self.auxiliary_security_header().unwrap().len();
        }

        if fc.information_elements_present() {
            if let Some(ie) = self.information_elements() {
                offset += ie.len();
            }
        }

        if self.buffer.as_ref().len() <= offset {
            return None;
        }

        Some(&self.buffer.as_ref()[offset..])
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> DataFrame<T> {
    /// Set the Frame Control field values in the buffer, based on the given
    /// [`FrameControlRepr`].
    pub fn set_frame_control(&mut self, fc: &FrameControlRepr) {
        let mut w = FrameControl::new_unchecked(&mut self.buffer.as_mut()[..2]);
        w.set_frame_type(fc.frame_type);
        w.set_security_enabled(fc.security_enabled);
        w.set_frame_pending(fc.frame_pending);
        w.set_ack_request(fc.ack_request);
        w.set_pan_id_compression(fc.pan_id_compression);
        w.set_sequence_number_suppression(fc.sequence_number_suppression);
        w.set_information_elements_present(fc.information_elements_present);
        w.set_dst_addressing_mode(fc.dst_addressing_mode);
        w.set_src_addressing_mode(fc.src_addressing_mode);
        w.set_frame_version(fc.frame_version);
    }

    /// Get a mutable reference to the Frame Control fields
    pub fn frame_control_mut(&mut self) -> FrameControl<&'_ mut [u8]> {
        FrameControl::new_unchecked(&mut self.buffer.as_mut()[..2])
    }

    /// Set the Sequence Number field value in the buffer.
    pub fn set_sequence_number(&mut self, sequence_number: u8) {
        // Set the sequence number suppression bit to false.
        let mut w = FrameControl::new_unchecked(&mut self.buffer.as_mut()[..2]);
        w.set_sequence_number_suppression(false);

        self.buffer.as_mut()[2] = sequence_number;
    }

    /// Set the Addressing field values in the buffer, based on the given
    /// [`AddressingFieldsRepr`].
    pub fn set_addressing_fields(&mut self, addressing_fields: &AddressingFieldsRepr) {
        let start = 2 + (!self.frame_control().sequence_number_suppression() as usize);

        let (fc, addressing) = self.buffer.as_mut().split_at_mut(start);
        let mut w = AddressingFields::new_unchecked(addressing, FrameControl::new_unchecked(fc));
        w.write_fields(addressing_fields);
    }

    /// Set the Auxiliary Security Header field values in the buffer, based on
    /// the given _.
    pub fn set_aux_sec_header(&mut self) {
        todo!();
    }

    /// Set the Information Elements field values in the buffer, based on the
    /// given _.
    pub fn set_information_elements(
        &mut self,
        ie: &InformationElementsRepr,
        contains_payload: bool,
    ) {
        let mut offset = 2;
        offset += !self.frame_control().sequence_number_suppression() as usize;

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        ie.emit(&mut self.buffer.as_mut()[offset..], contains_payload);
    }

    /// Set the payload of the frame.
    pub fn set_payload(&mut self, payload: &[u8]) {
        let mut offset = 0;
        offset += 2;

        if !self.frame_control().sequence_number_suppression() {
            offset += 1;
        }

        if let Some(af) = self.addressing() {
            offset += af.len();
        }

        if self.frame_control().security_enabled() {
            offset += self.auxiliary_security_header().unwrap().len();
        }

        if self.frame_control().information_elements_present() {
            offset += self.information_elements().unwrap().len();
        }

        self.buffer.as_mut()[offset..].copy_from_slice(payload);
    }
}
