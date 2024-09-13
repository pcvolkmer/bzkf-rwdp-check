/*
 * This file is part of bzkf-rwdp-check
 *
 * Copyright (C) 2024 the original author or authors.
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

SELECT CASE
           WHEN condcodingcode LIKE 'C00%'
               OR condcodingcode LIKE 'C01%'
               OR condcodingcode LIKE 'C02%'
               OR condcodingcode LIKE 'C03%'
               OR condcodingcode LIKE 'C04%'
               OR condcodingcode LIKE 'C05%'
               OR condcodingcode LIKE 'C06%'
               OR condcodingcode LIKE 'C07%'
               OR condcodingcode LIKE 'C08%'
               OR condcodingcode LIKE 'C09%'
               OR condcodingcode LIKE 'C10%'
               OR condcodingcode LIKE 'C11%'
               OR condcodingcode LIKE 'C12%'
               OR condcodingcode LIKE 'C13%'
               OR condcodingcode LIKE 'C14%' THEN 'C00-C14'

           WHEN condcodingcode LIKE 'C15%' THEN 'C15'

           WHEN condcodingcode LIKE 'C16%' THEN 'C16'

           WHEN condcodingcode LIKE 'C18%'
               OR condcodingcode LIKE 'C19%'
               OR condcodingcode LIKE 'C20%'
               OR condcodingcode LIKE 'C21%' THEN 'C18-C21'

           WHEN condcodingcode LIKE 'C22%' THEN 'C22'

           WHEN condcodingcode LIKE 'C23%'
               OR condcodingcode LIKE 'C24%' THEN 'C23-C24'

           WHEN condcodingcode LIKE 'C25%' THEN 'C25'

           WHEN condcodingcode LIKE 'C32%' THEN 'C32'

           WHEN condcodingcode LIKE 'C33%'
               OR condcodingcode LIKE 'C34%' THEN 'C33-C34'

           WHEN condcodingcode LIKE 'C43%' THEN 'C43'

           WHEN condcodingcode LIKE 'C50%'
               OR condcodingcode LIKE 'D05%' THEN 'C50, D05'

           WHEN condcodingcode LIKE 'C53%'
               OR condcodingcode LIKE 'D06%' THEN 'C53, D06'

           WHEN condcodingcode LIKE 'C54%'
               OR condcodingcode LIKE 'C55%' THEN 'C54-C55'

           WHEN condcodingcode LIKE 'C56%'
               OR condcodingcode = 'D39.1' THEN 'C56, D39.1'

           WHEN condcodingcode LIKE 'C61%' THEN 'C61'

           WHEN condcodingcode LIKE 'C62%' THEN 'C62'

           WHEN condcodingcode LIKE 'C64%' THEN 'C64'

           WHEN condcodingcode LIKE 'C67%'
               OR condcodingcode = 'D09.0'
               OR condcodingcode = 'D41.4' THEN 'C67, D09.0, D41.4'

           WHEN condcodingcode LIKE 'C70%'
               OR condcodingcode LIKE 'C71%'
               OR condcodingcode LIKE 'C72%' THEN 'C70-C72'

           WHEN condcodingcode LIKE 'C73%' THEN 'C73'

           WHEN condcodingcode LIKE 'C81%' THEN 'C81'

           WHEN condcodingcode LIKE 'C82%'
               OR condcodingcode LIKE 'C83%'
               OR condcodingcode LIKE 'C84%'
               OR condcodingcode LIKE 'C85%'
               OR condcodingcode LIKE 'C86%'
               OR condcodingcode LIKE 'C87%'
               OR condcodingcode LIKE 'C88%'
               OR condcodingcode LIKE 'C96%' THEN 'C82-C88, C96'

           WHEN condcodingcode LIKE 'C90%' THEN 'C90'

           WHEN condcodingcode LIKE 'C91%'
               OR condcodingcode LIKE 'C92%'
               OR condcodingcode LIKE 'C93%'
               OR condcodingcode LIKE 'C94%'
               OR condcodingcode LIKE 'C95%' THEN 'C91-C95'

           ELSE 'Other'
           END AS ICD10_GROUP,
       '' AS schema_version,
       COUNT(*) as COUNT
FROM (
    SELECT DISTINCT
        lme.lkr_meldung,
        EXTRACTVALUE(lme.xml_daten, '//Patienten_Stammdaten/@Patient_ID') AS pid,
        EXTRACTVALUE(lme.xml_daten, '//ADT_GEKID/@Schema_Version') AS schema_version,
        lme.versionsnummer,
        SHA2(CONCAT('https://fhir.diz.uk-erlangen.de/identifiers/onkostar-xml-condition-id|', EXTRACTVALUE(lme.xml_daten, '//Patienten_Stammdaten/@Patient_ID'), 'condition', EXTRACTVALUE(lme.xml_daten, '//Diagnose/@Tumor_ID')), 256) AS cond_id,
        SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Primaertumor_ICD_Code'), ' ', 1) AS condcodingcode,
        SUBSTRING_INDEX(SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Diagnosedatum'), ' ', 1), '.', -1) AS diagnosejahr
    FROM lkr_meldung_export lme
    WHERE lme.xml_daten LIKE '%ICD_Version%'
        AND lme.typ <> -1
        AND lme.xml_daten NOT LIKE '%<Menge_Tumorkonferenz%'
        AND SUBSTRING_INDEX(SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Diagnosedatum'), ' ', 1), '.', -1) = :year
        AND (lme.xml_daten NOT LIKE '%histologie_zytologie%' OR 1 = :include_histo_zyto)
        AND (EXTRACTVALUE(lme.xml_daten, '//Meldende_Stelle') NOT LIKE '%9999%' OR 1 <= :include_extern)
        AND (EXTRACTVALUE(lme.xml_daten, '//ADT_GEKID/@Schema_Version') LIKE '2.%' OR 1 = :ignore_non_obds_2)
 ) o1
LEFT OUTER JOIN (
    SELECT DISTINCT
        lme.lkr_meldung,
        SHA2(CONCAT('https://fhir.diz.uk-erlangen.de/identifiers/onkostar-xml-condition-id|', EXTRACTVALUE(lme.xml_daten, '//Patienten_Stammdaten/@Patient_ID'), 'condition', EXTRACTVALUE(lme.xml_daten, '//Diagnose/@Tumor_ID')), 256) AS cond_id,
        CASE WHEN STR_TO_DATE(EXTRACTVALUE(lme.xml_daten, '//Meldedatum'), '%d.%c.%Y') < :ignore_exports_since THEN MAX(versionsnummer) ELSE ~0 END AS max_version
    FROM lkr_meldung_export lme
    WHERE SUBSTRING_INDEX(SUBSTRING_INDEX(EXTRACTVALUE(lme.xml_daten, '//Diagnosedatum'), ' ', 1), '.', -1) = :year
    GROUP BY cond_id ORDER BY cond_id
) o2
ON (o1.cond_id = o2.cond_id AND o1.versionsnummer < max_version)
WHERE diagnosejahr = :year AND o2.cond_id IS NULL
GROUP BY ICD10_GROUP;