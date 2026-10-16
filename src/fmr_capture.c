/* One selected macOS driver-ownership capture for the FMR1 read-only test.
 * Capture is mode-changing, not passive inventory or transaction recovery.
 * The capture option terminates device/non-MSC interface drivers. The SDK
 * expressly excludes MSC interface drivers, so an ordinary claim may still
 * fail afterward. No flash operation or USB reset is requested here.
 */
#include <CoreFoundation/CoreFoundation.h>
#include <IOKit/IOCFPlugIn.h>
#include <IOKit/IOKitLib.h>
#include <IOKit/IOReturn.h>
#include <IOKit/usb/IOUSBLib.h>
#include <IOKit/usb/USB.h>
#include <limits.h>
#include <stdint.h>
#include <unistd.h>

static int number_property(io_service_t service, CFStringRef key, int64_t *out)
{
    CFTypeRef value = IORegistryEntryCreateCFProperty(service, key, kCFAllocatorDefault, 0);
    if (value == NULL) {
        return 0;
    }
    int ok = CFGetTypeID(value) == CFNumberGetTypeID() &&
        CFNumberGetValue((CFNumberRef)value, kCFNumberSInt64Type, out);
    CFRelease(value);
    return ok;
}

/* On success the returned COM interface is an opaque ownership lease. The
 * caller must retain it until all selected USB handles have been released.
 * On failure lease_out remains NULL; no retry or undo/reset is attempted.
 */
int32_t fmr_capture_selected(uint64_t registry_id, uint32_t location,
                             uint8_t address, void **lease_out)
{
    if (lease_out == NULL) {
        return (int32_t)kIOReturnBadArgument;
    }
    *lease_out = NULL;
    if (geteuid() != 0) {
        return (int32_t)kIOReturnNotPrivileged;
    }
    if (registry_id == 0 || location == 0 || address == 0 || address > 127) {
        return (int32_t)kIOReturnBadArgument;
    }
    CFMutableDictionaryRef matching = IORegistryEntryIDMatching(registry_id);
    if (matching == NULL) {
        return (int32_t)kIOReturnNoMemory;
    }
    /* Matching consumes the dictionary and obtains only this registry object. */
    io_service_t service = IOServiceGetMatchingService(kIOMainPortDefault, matching);
    if (service == IO_OBJECT_NULL) {
        return (int32_t)kIOReturnNoDevice;
    }
    uint64_t actual_id = 0;
    int64_t vendor = 0, product = 0, actual_location = 0, actual_address = 0;
    IOReturn result = IORegistryEntryGetRegistryEntryID(service, &actual_id);
    if (result != kIOReturnSuccess || actual_id != registry_id ||
        (!IOObjectConformsTo(service, "IOUSBHostDevice") &&
         !IOObjectConformsTo(service, "IOUSBDevice")) ||
        !number_property(service, CFSTR("idVendor"), &vendor) || vendor != 0x4c4a ||
        !number_property(service, CFSTR("idProduct"), &product) || product != 0x8057 ||
        !number_property(service, CFSTR("locationID"), &actual_location) ||
        actual_location < INT32_MIN || actual_location > UINT32_MAX ||
        (uint32_t)actual_location != location ||
        !number_property(service, CFSTR("USB Address"), &actual_address) ||
        actual_address != address) {
        IOObjectRelease(service);
        return (int32_t)kIOReturnBadArgument;
    }

    IOCFPlugInInterface **plugin = NULL;
    SInt32 score = 0;
    result = IOCreatePlugInInterfaceForService(service, kIOUSBDeviceUserClientTypeID,
                                              kIOCFPlugInInterfaceID, &plugin, &score);
    IOObjectRelease(service);
    if (result != kIOReturnSuccess || plugin == NULL) {
        if (plugin != NULL) {
            (*plugin)->Release(plugin);
        }
        return (int32_t)(result == kIOReturnSuccess ? kIOReturnError : result);
    }
    IOUSBDeviceInterface500 **device = NULL;
    HRESULT query = (*plugin)->QueryInterface(plugin,
        CFUUIDGetUUIDBytes(kIOUSBDeviceInterfaceID500), (LPVOID *)&device);
    (*plugin)->Release(plugin);
    if (query != S_OK || device == NULL) {
        if (device != NULL) {
            (*device)->Release(device);
        }
        return (int32_t)kIOReturnUnsupported;
    }
    /* Exactly one explicit capture, while closed. Current IOUSBLib.h permits
     * this root-only option without USBDeviceOpen/OpenSeize. Unlike ordinary
     * reenumeration, the capture branch is not a requested USB/device reset.
     * Do not pass zero, release-mask or any reset/configuration operation.
     */
    result = (*device)->USBDeviceReEnumerate(device, kUSBReEnumerateCaptureDeviceMask);
    if (result != kIOReturnSuccess) {
        (*device)->Release(device);
        return (int32_t)result;
    }
    *lease_out = device;
    return (int32_t)kIOReturnSuccess;
}

/* Drop only this COM reference. Never release capture back to the OS: that
 * would request reenumeration and could leave the selected ROM mode.
 */
void fmr_release_capture_object(void *lease)
{
    if (lease != NULL) {
        IOUSBDeviceInterface500 **device = (IOUSBDeviceInterface500 **)lease;
        (*device)->Release(device);
    }
}
