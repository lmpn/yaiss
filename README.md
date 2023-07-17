# Yet Another Image Storage Service

[![Coverage Status](https://coveralls.io/repos/github/lmpn/yaiss/badge.svg)](https://coveralls.io/github/lmpn/yaiss)
[![Build status](https://github.com/lmpn/yaiss/actions/workflows/pr-workflow.yml/badge.svg?branch=main)](https://github.com/lmpn/yaiss/actions/workflows/pr-workflow.yml)

This is an application that lets its user to store its images in lossless and compact format (QOI).

The objective is to try to provide a solution with good performance and storage.


## 1st feature set 
- Upload several images
    - Convert them to raw and then to qoi
- Delete several images
- Get several images

## 2nd feature set
- Tags
- Search
    - Tags
    - Time
    - Person  - requires 3rd fs
- Monitoring
    - Status code per req
    - Timing per req 

## 3rd feature set 
- Face recognition
    - Complete search by person
- Group/folders
- Duplicates removal
- NSFW - blur

## Extra
- Security
    - HTTPS 
- Account
- Compression
- Coverage