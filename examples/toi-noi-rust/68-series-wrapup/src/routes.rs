//! Ba đường đọc theo vai trò. Mỗi đường là danh sách số bài tăng dần.

pub struct Route {
    pub who: &'static str,
    pub goal: &'static str,
    pub articles: &'static [u8],
}

pub const ROUTES: &[Route] = &[
    Route {
        who: "Backend C# muốn đọc nhanh",
        goal: "hiểu quyết định, khác biệt cốt lõi với C# và hợp đồng giữa hai bên",
        articles: &[1, 3, 4, 7, 13, 24, 32, 35, 37, 55, 58, 59, 62, 66, 68],
    },
    Route {
        who: "Người mới học Rust",
        goal: "ngôn ngữ trước, rồi bắc cầu sang service",
        articles: &[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            31, 40,
        ],
    },
    Route {
        who: "Người chỉ cần sổ cái",
        goal: "nghiệp vụ và bất biến; đọc code Rust như mã giả",
        articles: &[13, 24, 25, 41, 42, 43, 46, 47, 48, 50, 54, 67, 68],
    },
];

/// Kiểm đường đọc: số bài tồn tại, tăng dần (không trùng).
pub fn check(routes: &[Route], article_count: u8) -> Vec<String> {
    let mut problems = Vec::new();
    for route in routes {
        if route.articles.windows(2).any(|w| w[0] >= w[1]) {
            problems.push(format!("đường đọc «{}»: số bài phải tăng dần", route.who));
        }
        if let Some(bad) = route
            .articles
            .iter()
            .find(|&&n| n == 0 || n > article_count)
        {
            problems.push(format!(
                "đường đọc «{}»: bài {bad} không tồn tại",
                route.who
            ));
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_routes_are_valid_and_end_sensibly() {
        assert!(check(ROUTES, 68).is_empty());
        assert_eq!(ROUTES.len(), 3);
        assert_eq!(ROUTES[0].articles.last(), Some(&68));
    }

    #[test]
    fn check_catches_descending_and_unknown() {
        let bad = [Route {
            who: "x",
            goal: "y",
            articles: &[5, 3, 70],
        }];
        let problems = check(&bad, 68);
        assert_eq!(problems.len(), 2);
    }
}
