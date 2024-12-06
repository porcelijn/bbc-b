#ifndef __INC_VIA_H
#define __INC_VIA_H

typedef struct state_t state_t;
void raise_interrupt(state_t* state, int level);

typedef struct VIA VIA;
typedef struct VIA
{
        uint8_t  ora,   orb,   ira,   irb;
        uint8_t  ddra,  ddrb;
        uint8_t  sr;
        uint8_t  t1pb7;
        uint32_t t1l,   t2l;
        int      t1c,   t2c;
        uint8_t  acr,   pcr,   ifr,   ier;
        int      t1hit, t2hit;
        int      ca1,   ca2,   cb1,   cb2;
        int      intnum;
        int      sr_count;

        uint8_t  (*read_portA)(state_t* state);
        uint8_t  (*read_portB)(state_t* state);
        void     (*write_portA)(state_t* state, uint8_t val);
        void     (*write_portB)(state_t* state, uint8_t val);

        void     (*set_ca1)(VIA* sysvia, int level);
        void     (*set_ca2)(VIA* sysvia, int level);
        void     (*set_cb1)(VIA* sysvia, int level);
        void     (*set_cb2)(VIA* sysvia, int level);
        void     (*timer_expire1)(void *);

        state_t* state;
} VIA;

uint8_t via_read(VIA *v, uint16_t addr);
void    via_write(VIA *v, uint16_t addr, uint8_t val);
void    via_reset(VIA *v);
void    via_shift(VIA *v, int cycles);

void via_set_ca1(VIA *v, int level);
void via_set_ca2(VIA *v, int level);
void via_set_cb1(VIA *v, int level);
void via_set_cb2(VIA *v, int level);

//void via_savestate(VIA *v, FILE *f);
//void via_loadstate(VIA *v, FILE *f);

void via_poll(VIA *v, int cycles);

#endif
