MEMORY
{
  /* NOTE K = KiBi = 1024 bytes */
  /* Softdevice S140 6.1.1 uses 0x26000 (152KB) of flash */
  FLASH : ORIGIN = 0x00026000, LENGTH = 872K
  /* Softdevice uses 0x2aa8 (~11KB) of RAM */
  RAM : ORIGIN = 0x20002aa8, LENGTH = 245K
}