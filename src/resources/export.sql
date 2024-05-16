/*
 * This file is part of bzkf-rwdp-check
 *
 * Copyright (C) 2024 Comprehensive Cancer Center Mainfranken and contributors.
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 */

SELECT
    o1.cond_id,
    o1.condcodingcode,
    o1.diagnosedatum,
    o1.pid
FROM (

    SELECT DISTINCT
        EXTRACTVALUE(lme.xml_daten, '//Patienten_Stammdaten/@Patient_ID') AS pid,
        lme.versionsnummer,
        SHA2(CONCAT('https://fhir.diz.uk-erlangen.de/identifiers/onkostar-xml-condition-id|', EXTRACTVALUE(lme.xml_daten, '//Patienten_Stammdaten/@Patient_ID'), 'condition', EXTRACTVALUE(lme.xml_daten, '//Diagnose/@Tumor_ID')), 256) AS cond_id,
        SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Primaertumor_ICD_Code'), ' ', 1) AS condcodingcode,
        SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Diagnosedatum'), ' ', 1) AS diagnosedatum,
        SUBSTRING_INDEX(SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Diagnosedatum'), ' ', 1), '.', -1) AS diagnosejahr
    FROM lkr_meldung_export lme
    JOIN lkr_meldung lm ON (lm.id = lme.lkr_meldung AND lme.typ <> '-1' AND lm.extern <= :include_extern)
    WHERE lme.xml_daten LIKE '%ICD_Version%'
        AND SUBSTRING_INDEX(SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Diagnosedatum'), ' ', 1), '.', -1) = :year
        AND (lme.xml_daten LIKE '%<cTNM%' OR lme.xml_daten LIKE '%<pTNM%' OR lme.xml_daten LIKE '%<Menge_Histologie>%' OR lme.xml_daten LIKE '%<Menge_Weitere_Klassifikation>%')
    ) o1
    LEFT OUTER JOIN (

    SELECT DISTINCT
        SHA2(CONCAT('https://fhir.diz.uk-erlangen.de/identifiers/onkostar-xml-condition-id|', EXTRACTVALUE(lme.xml_daten, '//Patienten_Stammdaten/@Patient_ID'), 'condition', EXTRACTVALUE(lme.xml_daten, '//Diagnose/@Tumor_ID')), 256) AS cond_id,
        CASE WHEN le.exportiert_am < :ignore_exports_since THEN MAX(versionsnummer) ELSE ~0 END AS max_version
    FROM lkr_meldung_export lme
        JOIN lkr_export le ON (lme.lkr_export = le.id)
    WHERE SUBSTRING_INDEX(SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Diagnosedatum'), ' ', 1), '.', -1) = :year
    GROUP BY cond_id ORDER BY cond_id

    ) o2
ON (o1.cond_id = o2.cond_id AND o1.versionsnummer < max_version)
WHERE diagnosejahr = :year AND o2.cond_id IS NULL;