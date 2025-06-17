@echo off

 
set src1=BByte_client.exe
set dest1=libs\Resource_client.bin

set src2=BByte_client_loader.exe
set dest2=libs\Resource_client_loader.bin

set src3=BByte_client_loader.exe
set dest3=libs\Resource_loader.bin

set src7=BByte_loader.exe
set dest7=libs\Resource_loader.bin

set src4=BByte_inject.exe
set dest4=libs\Resource_inject.bin

set src5=BByte_ftp.exe
set dest5=libs\Resource_ftp.bin

set src6=BByte_shell.exe
set dest6=libs\Resource_shell.bin

set src10=BByte_proxy.exe
set dest10=libs\Resource_proxy.bin

set src8=BByte.exe
set dest8=Bbyte.exe
 
move /Y %src7% %dest7%
move /Y %src1% %dest1%
 
move /Y %src2% %dest2%
move /Y %src3% %dest3%
 
move /Y %src4% %dest4%
move /Y %src5% %dest5%
 
move /Y %src6% %dest6%
move /Y %src8% %dest8%
move /Y %src10% %dest10%

echo Файлы успешно перемещены.
pause

