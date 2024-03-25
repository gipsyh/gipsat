#pragma once

#include "transys.h"

extern "C" {
void *gipsat_new(const void *);

void drop_gipsat(void *);
}

class GipSAT {
    public:
	GipSAT(Transys &transys)
	{
		ptr = gipsat_new(transys.ptr);
	}

	~GipSAT()
	{
		drop_gipsat(ptr);
	}

    private:
	void *ptr;
};