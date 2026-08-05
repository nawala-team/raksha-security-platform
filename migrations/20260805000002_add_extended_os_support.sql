-- Raksha Security Platform
-- Migration: Add extended OS support for enterprise environments
-- Date: 2026-08-05
-- 
-- Adds support for:
-- HIGH PRIORITY: Solaris, AIX (enterprise Unix)
-- MEDIUM PRIORITY: Alpine, Flatcar, Bottlerocket, CoreOS, Photon OS (cloud-native)

-- ============================================================
-- Update agents table: extend os_type CHECK constraint
-- ============================================================

-- Drop existing constraint and recreate with extended OS list
ALTER TABLE agents DROP CONSTRAINT IF EXISTS agents_os_type_check;

ALTER TABLE agents ADD CONSTRAINT agents_os_type_check 
    CHECK (os_type IN (
        -- Original supported OS
        'linux', 'windows', 'macos', 'freebsd', 'openbsd',
        -- Enterprise Unix (HIGH PRIORITY)
        'solaris', 'aix',
        -- Cloud-native / Container OS (MEDIUM PRIORITY)
        'alpine', 'flatcar', 'bottlerocket', 'coreos', 'photon',
        -- Additional BSD variants
        'netbsd', 'dragonflybsd'
    ));

-- Update comment to reflect supported OS list
COMMENT ON COLUMN agents.os_type IS 
'Operating system type. Supported: linux, windows, macos, freebsd, openbsd, solaris, aix, alpine, flatcar, bottlerocket, coreos, photon, netbsd, dragonflybsd';

-- ============================================================
-- Add os_family reference table for UI dropdowns and validation
-- ============================================================

CREATE TABLE IF NOT EXISTS os_families (
    id              VARCHAR(30) PRIMARY KEY,
    display_name    VARCHAR(100) NOT NULL,
    category        VARCHAR(30) NOT NULL CHECK (category IN (
        'linux', 'windows', 'macos', 'bsd', 'unix', 'container_os'
    )),
    vendor          VARCHAR(100),
    icon            VARCHAR(50),
    is_active       BOOLEAN NOT NULL DEFAULT true,
    sort_order      INTEGER NOT NULL DEFAULT 100,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert all supported OS families
INSERT INTO os_families (id, display_name, category, vendor, icon, sort_order) VALUES
    -- Linux distributions
    ('linux', 'Linux', 'linux', 'Various', 'linux', 10),
    ('ubuntu', 'Ubuntu', 'linux', 'Canonical', 'ubuntu', 11),
    ('debian', 'Debian', 'linux', 'Debian Project', 'debian', 12),
    ('rhel', 'Red Hat Enterprise Linux', 'linux', 'Red Hat', 'redhat', 13),
    ('centos', 'CentOS', 'linux', 'CentOS Project', 'centos', 14),
    ('rocky', 'Rocky Linux', 'linux', 'Rocky Enterprise', 'rocky', 15),
    ('alma', 'AlmaLinux', 'linux', 'AlmaLinux OS Foundation', 'alma', 16),
    ('fedora', 'Fedora', 'linux', 'Fedora Project', 'fedora', 17),
    ('suse', 'SUSE Linux Enterprise', 'linux', 'SUSE', 'suse', 18),
    ('opensuse', 'openSUSE', 'linux', 'openSUSE Project', 'opensuse', 19),
    ('arch', 'Arch Linux', 'linux', 'Arch Linux', 'arch', 20),
    ('gentoo', 'Gentoo', 'linux', 'Gentoo Foundation', 'gentoo', 21),
    ('amazon', 'Amazon Linux', 'linux', 'Amazon', 'amazon', 22),
    ('oracle_linux', 'Oracle Linux', 'linux', 'Oracle', 'oracle', 23),
    
    -- Cloud-native / Container OS (MEDIUM PRIORITY)
    ('alpine', 'Alpine Linux', 'container_os', 'Alpine Linux', 'alpine', 30),
    ('flatcar', 'Flatcar Container Linux', 'container_os', 'Kinvolk/Microsoft', 'flatcar', 31),
    ('bottlerocket', 'Bottlerocket', 'container_os', 'Amazon', 'bottlerocket', 32),
    ('coreos', 'CoreOS', 'container_os', 'Red Hat', 'coreos', 33),
    ('photon', 'VMware Photon OS', 'container_os', 'VMware', 'photon', 34),
    ('talos', 'Talos Linux', 'container_os', 'Sidero Labs', 'talos', 35),
    ('rancher', 'RancherOS', 'container_os', 'Rancher/SUSE', 'rancher', 36),
    
    -- Windows
    ('windows', 'Windows', 'windows', 'Microsoft', 'windows', 40),
    ('windows_server', 'Windows Server', 'windows', 'Microsoft', 'windows', 41),
    
    -- macOS
    ('macos', 'macOS', 'macos', 'Apple', 'apple', 50),
    
    -- BSD family
    ('freebsd', 'FreeBSD', 'bsd', 'FreeBSD Foundation', 'freebsd', 60),
    ('openbsd', 'OpenBSD', 'bsd', 'OpenBSD Project', 'openbsd', 61),
    ('netbsd', 'NetBSD', 'bsd', 'NetBSD Foundation', 'netbsd', 62),
    ('dragonflybsd', 'DragonFly BSD', 'bsd', 'DragonFly BSD Project', 'dragonfly', 63),
    
    -- Enterprise Unix (HIGH PRIORITY)
    ('solaris', 'Oracle Solaris', 'unix', 'Oracle', 'solaris', 70),
    ('illumos', 'illumos', 'unix', 'illumos Project', 'illumos', 71),
    ('aix', 'IBM AIX', 'unix', 'IBM', 'ibm', 72),
    ('hpux', 'HP-UX', 'unix', 'Hewlett Packard Enterprise', 'hpe', 73)
ON CONFLICT (id) DO NOTHING;

-- Index for category filtering
CREATE INDEX IF NOT EXISTS idx_os_families_category ON os_families(category);
CREATE INDEX IF NOT EXISTS idx_os_families_active ON os_families(is_active) WHERE is_active = true;

COMMENT ON TABLE os_families IS 'Reference table for supported operating system families with metadata for UI display';
