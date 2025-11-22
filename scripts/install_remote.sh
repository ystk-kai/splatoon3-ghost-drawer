#!/bin/bash
set -x

echo "Stopping service..."
systemctl stop splatoon3-ghost-drawer.service || true

echo "Killing process..."
pkill -f splatoon3-ghost-drawer || true

echo "Removing old binary..."
rm -f /usr/local/bin/splatoon3-ghost-drawer
rm -f /opt/splatoon3-ghost-drawer/splatoon3-ghost-drawer

echo "Moving new binary..."
mv /tmp/splatoon3-ghost-drawer /usr/local/bin/
chmod +x /usr/local/bin/splatoon3-ghost-drawer

echo "Running setup..."
/usr/local/bin/splatoon3-ghost-drawer setup --force

echo "Restarting service..."
systemctl restart splatoon3-ghost-drawer.service
systemctl status splatoon3-ghost-drawer.service
