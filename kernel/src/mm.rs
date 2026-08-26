pub mod physical;
pub mod mmdat;

use core::{ptr::{self, NonNull},
           ops::{Add, Sub}};

use spin::Once;

use bytemuck::AnyBitPattern;

static HHDM_START: Once<VirtAddr> = Once::new();

#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AnyBitPattern)]
pub struct PhysAddr(usize);


impl PhysAddr {
    ///
    /// This routine converts a physical address to a virtual pointer using HHDM.
    ///
    pub fn as_hhdm<T>(self) -> *mut T {
        let hhdm_virt = *HHDM_START.get().expect("HHDM not initialized yet!"); 
        VirtAddr(self.0 + hhdm_virt.0).as_ptr()
    }

    ///
    /// This routine zeros out memory at a physical address via HHDM.
    ///
    #[allow(dead_code)]
    pub(crate) fn zero_hhdm(self, len: usize) {
        unsafe { ptr::write_bytes(self.as_hhdm::<u8>(), 0, len) };
    }
}

#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, AnyBitPattern)]
pub struct VirtAddr(usize);

impl VirtAddr {
    ///
    /// This routine converts a virtual address to a mutable pointer.
    ///
    pub fn as_ptr<T>(self) -> *mut T {
        ptr::with_exposed_provenance_mut(self.0)
    }

    ///
    /// This routine converts a virtual address to a physical address via HHDM offset.
    ///
    pub fn as_hhdm(self) -> Option<PhysAddr> {
        let hhdm_virt = *HHDM_START.get()?; 
        self.0.checked_sub(hhdm_virt.0).map(PhysAddr)
    }
}

macro_rules! addr_impl {
    ($ty:ty) => {
        impl $ty {
            ///
            /// This routine returns a null address.
            ///
            #[inline]
            pub const fn null() -> Self {
                Self(0)
            }

            ///
            /// This routine creates a new address from a usize value.
            ///
            pub const fn new(value: usize) -> Self {
                Self(value)
            }

            ///
            /// This routine returns the raw usize value of the address.
            ///
            #[inline]
            pub const fn value(&self) -> usize {
                self.0
            }

            ///
            /// This routine checks if the address is null.
            ///
            #[inline]
            pub const fn is_null(&self) -> bool {
                self.0 == 0
            }
        }

        impl From<usize> for $ty {
            fn from(addr: usize) -> Self {
                Self(addr)
            }
        }

        #[cfg(target_pointer_width = "32")]
        impl From<u32> for $ty {
            fn from(addr: u32) -> Self {
                Self(addr as usize)
            }
        }

        #[cfg(target_pointer_width = "32")]
        impl Into<u32> for $ty {
            fn into(self) -> u32 {
                self.0 as u32
            }
        }

        #[cfg(target_pointer_width = "64")]
        impl From<u64> for $ty {
            fn from(addr: u64) -> Self {
                Self(addr as usize)
            }
        }

        #[cfg(target_pointer_width = "64")]
        impl From<$ty> for u64 {
            fn from(addr: $ty) -> u64 {
                addr.0 as u64
            }
        }

        impl<T> From<*const T> for $ty {
            fn from(ptr: *const T) -> Self {
                Self(ptr as usize)
            }
        }

        impl<T> From<*mut T> for $ty {
            fn from(ptr: *mut T) -> Self {
                Self(ptr as usize)
            }
        }

        impl<T> From<NonNull<T>> for $ty {
            fn from(ptr: NonNull<T>) -> Self {
                Self(ptr.as_ptr() as usize)
            }
        }

        impl Add for $ty {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl Sub for $ty {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl Add<usize> for $ty {
            type Output = Self;

            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 + rhs)
            }
        }

        impl Sub<usize> for $ty {
            type Output = Self;

            fn sub(self, rhs: usize) -> Self::Output {
                Self(self.0 - rhs)
            }
        }

        impl core::fmt::Debug for $ty {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("{:#x}", self.0))
            }
        }
    };
}

addr_impl!(PhysAddr);
addr_impl!(VirtAddr);
