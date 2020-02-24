/*
 * Copyright (C) 2018 bzt (bztsrc@github)
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy,
 * modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
 * WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 *
 */

#include "uart.h"
#include "mbox.h"

/**
 * Set screen resolution to 1024x768
 */
int lfb_init(unsigned int *size_phy_x, unsigned int *size_phy_y,
             unsigned int *size_virt_x, unsigned int *size_virt_y,
             unsigned int *offset_x, unsigned int *offset_y,
             unsigned int *depth,
             unsigned int *pitch,
             unsigned int *ptr)
{
    mbox[0] = 35*4;
    mbox[1] = MBOX_REQUEST;

    mbox[2] = 0x48003; //set phy wh
    mbox[3] = 8;
    mbox[4] = 8;
    mbox[5] = *size_phy_x; //FrameBufferInfo.width
    mbox[6] = *size_phy_y; //FrameBufferInfo.height

    mbox[7] = 0x48004; //set virt wh
    mbox[8] = 8;
    mbox[9] = 8;
    mbox[10] = *size_virt_x; //FrameBufferInfo.virtual_width
    mbox[11] = *size_virt_y; //FrameBufferInfo.virtual_height

    mbox[12] = 0x48009; //set virt offset
    mbox[13] = 8;
    mbox[14] = 8;
    mbox[15] = *offset_x; //FrameBufferInfo.x_offset
    mbox[16] = *offset_y; //FrameBufferInfo.y.offset

    mbox[17] = 0x48005; //set depth
    mbox[18] = 4;
    mbox[19] = 4;
    mbox[20] = *depth;  //FrameBufferInfo.depth

    mbox[21] = 0x48006; //set pixel order
    mbox[22] = 4;
    mbox[23] = 4;
    mbox[24] = 1;       //RGB, not BGR preferably

    mbox[25] = 0x40001; //get framebuffer, gets alignment on request
    mbox[26] = 8;
    mbox[27] = 8;
    mbox[28] = 4096;    //FrameBufferInfo.pointer
    mbox[29] = 0;       //FrameBufferInfo.size

    mbox[30] = 0x40008; //get pitch
    mbox[31] = 4;
    mbox[32] = 4;
    mbox[33] = 0;       //FrameBufferInfo.pitch

    mbox[34] = MBOX_TAG_LAST;

    //this might not return exactly what we asked for, could be
    //the closest supported resolution instead
    if(mbox_call(MBOX_CH_PROP) && mbox[20]==32 && mbox[28]!=0) {
//        mbox[28]&=0x3FFFFFFF;   //convert GPU address to ARM address
        *size_phy_x  = mbox[ 5]; // get actual physical width
        *size_phy_y  = mbox[ 6]; // get actual physical height
        *size_virt_x = mbox[10];
        *size_virt_y = mbox[11];
        *offset_x    = mbox[15];
        *offset_y    = mbox[16];
        *depth       = mbox[20];
        *pitch       = mbox[33]; // get number of bytes per line
//        isrgb=mbox[24];         //get the actual channel order
        *ptr         = mbox[28] & 0x3FFFFFFF; // convert GPU address to ARM address
        return 1;
    } else {
        return 0;
    }

    return 0;
}
