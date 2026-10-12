#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

// Runtime-loaded Syphon: builds without vendoring the third-party framework.
@interface NSObject (SelahSyphon)
- (id)initWithName:(NSString *)name device:(id<MTLDevice>)device options:(NSDictionary *)options;
- (void)publishFrameTexture:(id<MTLTexture>)texture onCommandBuffer:(id<MTLCommandBuffer>)buffer imageRegion:(NSRect)region flipped:(BOOL)flipped;
- (void)stop;
- (BOOL)hasClients;
@end
@interface SelahSyphonHandle : NSObject
@property(strong) id server;
@property(strong) id<MTLDevice> device;
@property(strong) id<MTLCommandQueue> queue;
@property(strong) id<MTLTexture> texture;
@end
@implementation SelahSyphonHandle
@end

void *selah_syphon_create(const char *name) {
 @autoreleasepool {
  NSArray *paths=@[@"/Library/Frameworks/Syphon.framework",[NSHomeDirectory() stringByAppendingPathComponent:@"Library/Frameworks/Syphon.framework"],[[NSBundle mainBundle].privateFrameworksPath stringByAppendingPathComponent:@"Syphon.framework"]?:@""];
  for(NSString *path in paths){NSBundle *bundle=[NSBundle bundleWithPath:path];if(bundle)[bundle load];}
  Class cls=NSClassFromString(@"SyphonMetalServer");if(!cls)return NULL;
  SelahSyphonHandle *handle=[SelahSyphonHandle new];handle.device=MTLCreateSystemDefaultDevice();if(!handle.device)return NULL;
  handle.queue=[handle.device newCommandQueue];handle.server=[[cls alloc] initWithName:[NSString stringWithUTF8String:name] device:handle.device options:nil];if(!handle.server)return NULL;
  return (__bridge_retained void *)handle;
 }
}
int selah_syphon_publish(void *pointer,unsigned width,unsigned height,const unsigned char *rgba) {
 @autoreleasepool {
  SelahSyphonHandle *h=(__bridge SelahSyphonHandle *)pointer;if(!h)return 0;
  if(!h.texture||h.texture.width!=width||h.texture.height!=height){MTLTextureDescriptor *d=[MTLTextureDescriptor texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm width:width height:height mipmapped:NO];d.usage=MTLTextureUsageShaderRead;d.storageMode=MTLStorageModeShared;h.texture=[h.device newTextureWithDescriptor:d];}
  if(!h.texture)return 0;
  [h.texture replaceRegion:MTLRegionMake2D(0,0,width,height) mipmapLevel:0 withBytes:rgba bytesPerRow:width*4];
  id<MTLCommandBuffer> buffer=[h.queue commandBuffer];if(!buffer)return 0;
  [h.server publishFrameTexture:h.texture onCommandBuffer:buffer imageRegion:NSMakeRect(0,0,width,height) flipped:NO];[buffer commit];[buffer waitUntilCompleted];return buffer.status!=MTLCommandBufferStatusError;
 }
}
int selah_syphon_clients(void *pointer){@autoreleasepool{return [((__bridge SelahSyphonHandle *)pointer).server hasClients]?1:0;}}
void selah_syphon_destroy(void *pointer){@autoreleasepool{SelahSyphonHandle *h=(__bridge_transfer SelahSyphonHandle *)pointer;[h.server stop];}}
