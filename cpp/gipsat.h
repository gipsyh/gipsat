#pragma once

#include "transys.h"

extern "C" {
void *gipsat_new(const void *);

void gipsat_drop(void *);

void gipsat_extend(void *);

int gipsat_propagate(void *);
}

class GipSAT {
    public:
	GipSAT(Transys &transys)
	{
		ptr = gipsat_new(transys.ptr);
	}

	~GipSAT()
	{
		gipsat_drop(ptr);
	}

	void extend()
	{
		gipsat_extend(ptr);
	}

	bool propagate()
	{
		return gipsat_propagate(ptr) == 1;
	}

    private:
	void *ptr;
};