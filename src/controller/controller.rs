use std::slice::{from_raw_parts, from_raw_parts_mut};
use std::sync::{Arc, Mutex};
use super::super::bindings::{ws2811_fini, ws2811_render, ws2811_t, ws2811_wait};
use super::super::util::{RawColor, Result};

/// The main struct used to control lights.  Provides ways of
/// accessing the light color values and rendering those values to
/// the string.
#[derive(Clone, Debug)]
pub struct Controller {
    c_struct: Arc<Mutex<ws2811_t>>,
}

impl Controller {
    /// Creates a new Controller
    ///
    /// Note: This is only to be called from the Builder struct
    pub fn new(c_struct: ws2811_t) -> Self {
        Controller { c_struct: Arc::new(Mutex::new(c_struct)) }
    }

    /// Render the colors to the string.
    ///
    /// It doesn't automatically do this because it
    /// is a somewhat costly operation that should
    /// be batched.
    pub fn render(&mut self) -> Result<()> {
        let mut lock = self.c_struct.lock().unwrap();
        unsafe {
            let result: Result<()> = ws2811_render(&mut *lock).into();
            match result {
                Ok(_) => Ok(()),
                Err(e) => return Err(e),
            }
        }
        /*
        unsafe {
            return ws2811_render(&mut self.c_struct).into();
        }
        */
    }

    /// Wait for a render to be completed.
    pub fn wait(&mut self) -> Result<()> {
        let mut lock = self.c_struct.lock().unwrap();
        unsafe {
            let result: Result<()> = ws2811_wait(&mut *lock).into();
            match result {
                Ok(_) => Ok(()),
                Err(e) => return Err(e),
            }
        }

        /*
        unsafe {
            return ws2811_wait(&mut self.c_struct).into();
        }
        */
    }

    /// Gets the channels with non-zero number of LED's associated with them.
    ///
    /// I know this is somewhat non-intuitive, but naming it something like
    /// `active_channels(&self)` seemed overly verbose.
    pub fn channels(&self) -> Vec<usize> {
        let lock = self.c_struct.lock().unwrap();
        (0..lock.channel.len())
            .filter(|&x| lock.channel[x].count > 0)
            .collect::<Vec<_>>()
        /*
        (0..self.c_struct.channel.len())
            .filter(|x: _| self.c_struct.channel[x.clone()].count > 0)
            .collect::<Vec<_>>()
        */
    }

    /// Gets the brightness of the LEDs
    pub fn brightness(&self, channel: usize) -> u8 {
        let lock = self.c_struct.lock().unwrap();
        lock.channel[channel].brightness
    }

    /// Sets the brighness of the LEDs
    pub fn set_brightness(&mut self, channel: usize, value: u8) {
        let mut lock = self.c_struct.lock().unwrap();
        lock.channel[channel].brightness = value;
    }

    /// Gets a slice view to the color array to be written to the LEDs.
    /// See `leds_mut` for a mutable slice view to this data.
    ///
    /// # Safety
    /// This function is moderately unsafe because we rely on the promise
    /// from the C library that it will stick to its memory layout and that
    /// the pointer is valid.
    pub fn leds(&self, channel: usize) -> &[RawColor] {
        /*
         * Using unsafe here because we want to construct a slice
         * from just the raw pointer and the supposed number of elements
         * which is safe as long as our friends in "C land" hold to their
         * memory layout and we use a data type with compatible layout.
         */
        let lock = self.c_struct.lock().unwrap();
        unsafe {
            from_raw_parts(
                lock.channel[channel].leds as *const RawColor,
                lock.channel[channel].count as usize,
            )
        }
    }

    /// Gets a mutable slice pointing to the color array to be written to
    /// the LEDs.
    ///
    /// # Safety
    /// This function is moderately unsafe because we rely on the promise
    /// from the C library that it will stick to its memory layout and that
    /// the pointer is valid.
    pub fn leds_mut(&mut self, channel: usize) -> &mut [RawColor] {
        /*
         * Using unsafe here because we want to construct a slice
         * from just the raw pointer and the supposed number of elements
         * which is safe as long as our friends in "C land" hold to their
         * memory layout and we use a data type with compatible layout.
         */
        let lock = self.c_struct.lock().unwrap();
        unsafe {
            from_raw_parts_mut(
                lock.channel[channel].leds as *mut RawColor,
                lock.channel[channel].count as usize,
            )
        }
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        /*
         * Unsafe used here because we need to call an externed
         * function during the drop process.  Unfortunately,
         * I don't have a better way of dealing with this.
         */
        let mut lock = self.c_struct.lock().unwrap();
        unsafe {
            ws2811_fini(&mut *lock);
        }
    }
}
