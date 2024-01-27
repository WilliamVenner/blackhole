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
            if (exception.reason == NULL) {
                return strdup([exception.name UTF8String]);
            } else {
                return strdup([exception.reason UTF8String]);
            }
        }
    }
}

const char* objc_add_blackhole_to_favorites(const char* blackhole_path_ptr)
{
    @autoreleasepool {
        @try {
            CFURLRef url = (CFURLRef)[NSURL fileURLWithPath:[NSString stringWithUTF8String:blackhole_path_ptr]];

            LSSharedFileListRef favoriteItems = LSSharedFileListCreate(NULL, kLSSharedFileListFavoriteItems, NULL);
            if (favoriteItems) {
                LSSharedFileListItemRef item = LSSharedFileListInsertItemURL(favoriteItems, kLSSharedFileListItemLast, NULL, NULL, url, NULL, NULL);
                if (item) {
                    CFRelease(item);
                }
            }

            CFRelease(favoriteItems);

            return NULL;
        } @catch (NSException* exception) {
            if (exception.reason == NULL) {
                return strdup([exception.name UTF8String]);
            } else {
                return strdup([exception.reason UTF8String]);
            }
        }
    }
}