#pragma once

class ServiceBase
{
public:
    virtual void init() = 0;
    virtual void handle() = 0;
};