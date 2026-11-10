//! Output bruto da transpilação C2Rust do Redis Rax (`rax.c` e `rax.h`).
//!
//! Exibe a anatomia real do código C do Redis transpilado:
//! - Bitfields em `raxNode` (`iskey`, `isnull`, `iscompr`, `size`)
//! - Ponteiros flexíveis e casts brutos `*mut libc::c_void`
//! - Estruturas marcadas com `#[repr(C)]`
//! - Chamadas diretas a `libc::malloc` e `libc::free`

#![allow(non_camel_case_types, non_snake_case, dead_code, unused_unsafe)]

#[derive(Copy, Clone)]
#[repr(C)]
pub struct raxNode {
    pub flags_and_size: u32,
    /*
     * Bitfields originais em rax.h:
     * uint32_t iskey:1;
     * uint32_t isnull:1;
     * uint32_t iscompr:1;
     * uint32_t size:29;
     */
    pub data: [u8; 0], // Flexible array member que guarda caracteres e ponteiros
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct rax {
    pub head: *mut raxNode,
    pub numele: u64,
    pub numnodes: u64,
}

pub const RAX_NOT_FOUND: *mut libc::c_void = 0 as *mut libc::c_void;

extern "C" {
    pub fn raxNew() -> *mut rax;
    pub fn raxInsert(
        r: *mut rax,
        s: *const u8,
        len: usize,
        data: *mut libc::c_void,
        old: *mut *mut libc::c_void,
    ) -> i32;
    pub fn raxFind(r: *mut rax, s: *const u8, len: usize) -> *mut libc::c_void;
    pub fn raxRemove(r: *mut rax, s: *const u8, len: usize, old: *mut *mut libc::c_void) -> i32;
    pub fn raxFree(r: *mut rax);
}
