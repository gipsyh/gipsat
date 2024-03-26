#pragma once

#include "transys.h"
#include "giputils.h"

extern "C" {
void *gipsat_new(const void *);

void gipsat_drop(void *);

size_t gipsat_level(void *);

void gipsat_extend(void *);

void gipsat_add_lemma(void *, uint, uint *, uint);

int gipsat_propagate(void *);

class RustVec *gipsat_get_bad(void *);
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

	size_t level()
	{
		return gipsat_level(ptr);
	}

	void extend()
	{
		gipsat_extend(ptr);
	}

	void add_lemma(uint frame, std::vector<uint> &cube)
	{
		gipsat_add_lemma(ptr, frame, cube.data(), cube.size());
	}

	bool propagate()
	{
		return gipsat_propagate(ptr) == 1;
	}

	std::vector<uint> get_bad()
	{
		RustVec rv = gipsat_get_bad(ptr);
		std::vector<uint> res;
		uint *data = rv.data();
		for (int i = 0; i < rv.size(); ++i) {
			res.push_back(*(data + i));
		}
		return res;
	}

    private:
	void *ptr;
};