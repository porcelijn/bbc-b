#ifndef __INC_SYSVIA_H
#define __INC_SYSVIA_H

extern VIA sysvia;

typedef void* port_t;
typedef struct state_t state_t;

void    sysvia_reset(state_t * s);
void    sysvia_write(uint16_t addr, uint8_t val);
uint8_t sysvia_read(uint16_t addr);

//void    sysvia_savestate(FILE *f);
//void    sysvia_loadstate(FILE *f);

//extern uint8_t IC32;
//extern uint8_t sdbval;
//extern int scrsize;

void sysvia_set_ca1(port_t portA, int level);
void sysvia_set_ca2(port_t portA, int level);
void sysvia_set_cb1(port_t portB, int level);
void sysvia_set_cb2(port_t portB, int level);

#endif
