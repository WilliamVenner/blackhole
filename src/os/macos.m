@import AppKit;

#include <stdint.h>

const char* objc_set_blackhole_icon(const char* blackhole_path_ptr, const char* icon_path_ptr)
{
    @autoreleasepool {
        @try {
            NSString* icon_path = [NSString stringWithUTF8String:icon_path_ptr];
            NSImage* image = [[NSImage alloc] initByReferencingFile:icon_path];

            NSString* blackhole_path = [NSString stringWithUTF8String:blackhole_path_ptr];
            [[NSWorkspace sharedWorkspace] setIcon:image forFile:blackhole_path options:0];
            return NULL;
        } @catch (NSException* exception) {
            return strdup([exception.reason UTF8String]);
        }
    }
}