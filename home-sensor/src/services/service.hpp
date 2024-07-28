#pragma once

class ServiceBase
{
protected:
    virtual void handle_inner(ulong* start_ms) = 0;

public:
    virtual void init() = 0;
    // returns completion time ms
    ulong handle()
    {
        ulong start = millis();
        handle_inner(&start);
        ulong end = millis();
        return end - start;
    }
};