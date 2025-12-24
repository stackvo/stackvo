<?php
/**
 * Network and DNS utilities
 */

/**
 * Check if a domain is configured in DNS/hosts
 * 
 * @param string $domain Domain name to check
 * @return bool True if configured, false otherwise
 */
function isDomainConfigured($domain) {
    if (empty($domain)) {
        return false;
    }
    
    // Use gethostbyname to check if domain resolves
    // If it doesn't resolve, it returns the domain name itself
    $ip = gethostbyname($domain);
    
    // If gethostbyname returns the same string, domain is not configured
    // If it returns an IP, domain is configured
    return $ip !== $domain;
}
