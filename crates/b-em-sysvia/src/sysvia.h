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

void sysvia_set_ca1(VIA* sysvia, int level);
void sysvia_set_ca2(VIA* sysvia, int level);
void sysvia_set_cb1(VIA* sysvia, int level);
void sysvia_set_cb2(VIA* sysvia, int level);

#endif
