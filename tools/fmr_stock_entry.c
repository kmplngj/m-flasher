/* Host-only macOS qualification helper. Never invoked by tests or the app.
 * One selected, fixed V15 RAM-mailbox/reset request; no OTA or flash payload.
 * The new ROM object and later actual cold return must be observed separately.
 */
#include <CoreMIDI/CoreMIDI.h>
#include <CoreFoundation/CoreFoundation.h>
#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv)
{
    if (argc == 2 && strcmp(argv[1], "--describe") == 0) {
        puts("V15 logical SysEx: F0 22 24 35 7D F7 (one packet, no retry)");
        puts("Expected cable-0 USB-MIDI bytes: 04 F0 22 24 07 35 7D F7");
        return 0;
    }
    if (argc != 3 || strcmp(argv[1], "--send-once") != 0) {
        fputs("usage: fmr-stock-entry --describe | --send-once EXACT_MIDI_DESTINATION_ID\n", stderr);
        return 2;
    }
    char *end = NULL;
    errno = 0;
    long selected = strtol(argv[2], &end, 10);
    char canonical[32];
    snprintf(canonical, sizeof canonical, "%ld", selected);
    if (errno || !end || *end || selected == 0 || selected < INT_MIN || selected > INT_MAX || strcmp(canonical, argv[2]) != 0) {
        fputs("invalid exact destination ID\n", stderr);
        return 2;
    }
    MIDIEndpointRef destination = 0;
    ItemCount count = MIDIGetNumberOfDestinations();
    if (count > 256) { fputs("too many destinations\n", stderr); return 1; }
    for (ItemCount index = 0; index < count; ++index) {
        MIDIEndpointRef candidate = MIDIGetDestination(index);
        SInt32 identifier = 0;
        if (MIDIObjectGetIntegerProperty(candidate, kMIDIPropertyUniqueID, &identifier) != noErr || identifier != selected) { continue; }
        if (destination) { fputs("ambiguous exact destination ID\n", stderr); return 1; }
        destination = candidate;
    }
    if (!destination) { fputs("selected destination absent\n", stderr); return 1; }
    CFStringRef name = NULL;
    if (MIDIObjectGetStringProperty(destination, kMIDIPropertyName, &name) != noErr || !name) {
        fputs("selected endpoint name unavailable\n", stderr); return 1;
    }
    int expected_name = CFStringCompare(name, CFSTR("FM-1"), 0) == kCFCompareEqualTo;
    CFRelease(name);
    MIDIEntityRef entity = 0;
    MIDIDeviceRef device = 0;
    if (!expected_name || MIDIEndpointGetEntity(destination, &entity) != noErr || !entity || MIDIEntityGetDevice(entity, &device) != noErr || !device) {
        fputs("selected endpoint is not the expected physical FM-1 endpoint\n", stderr); return 1;
    }
    MIDIClientRef client = 0;
    MIDIPortRef port = 0;
    OSStatus status = MIDIClientCreate(CFSTR("FMR1 one-shot stock entry"), NULL, NULL, &client);
    if (status != noErr) { fprintf(stderr, "client failed: %d\n", (int)status); return 1; }
    status = MIDIOutputPortCreate(client, CFSTR("FMR1 selected stock entry"), &port);
    if (status != noErr) { MIDIClientDispose(client); fprintf(stderr, "port failed: %d\n", (int)status); return 1; }
    static const Byte request[6] = {0xF0, 0x22, 0x24, 0x35, 0x7D, 0xF7};
    MIDIPacketList packets;
    MIDIPacket *packet = MIDIPacketListInit(&packets);
    if (!MIDIPacketListAdd(&packets, sizeof packets, packet, 0, sizeof request, request)) {
        MIDIPortDispose(port); MIDIClientDispose(client); fputs("packet construction failed\n", stderr); return 1;
    }
    status = MIDISend(port, destination, &packets);
    printf("Fixed V15 entry send result: %d; no retry.\n", (int)status);
    /* MIDISend queues output; do not dispose the port immediately afterward. */
    if (status == noErr) { usleep(1000000); }
    MIDIPortDispose(port);
    MIDIClientDispose(client);
    return status == noErr ? 0 : 1;
}
